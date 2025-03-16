use crate::config::{Command, CommandType, Config};
use crate::http::HttpMethod;
use clap::{Args, ValueHint};
use convert_case::{Case, Casing};
use openapiv3::{
    OpenAPI, Operation, Parameter, PathItem, ReferenceOr, RequestBody, Schema, SchemaKind, Type,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fs;

#[derive(Args, Debug)]
pub struct ImportArguments {
    #[arg(value_hint = ValueHint::FilePath)]
    file: String,
}

pub fn init(args: ImportArguments) -> Result<(), Box<dyn std::error::Error>> {
    // Read the OpenAPI spec from a file
    let spec_content = fs::read_to_string(args.file)?;
    let spec: OpenAPI = serde_yaml::from_str(&spec_content)?;

    // Generate configuration
    let config = generate_config(&spec)?;

    println!("{:?}", config);

    config.save().expect("could not create config file");
    Ok(())
}

fn generate_config(spec: &OpenAPI) -> Result<Config, Box<dyn Error>> {
    let mut config = Config {
        envs: HashMap::new(),
        commands: HashMap::new(),
    };
    // Extract server URL
    let api_url = if let Some(server) = spec.servers.first() {
        server.url.clone()
    } else {
        "".to_string()
    };

    // Create environment configuration
    config.envs.insert(
        "prod".to_string(),
        HashMap::from([("API_URL".to_string(), api_url.to_string())]),
    );

    // Group operations by tag
    let mut tag_operations: HashMap<String, Vec<(&String, &PathItem, Operation)>> = HashMap::new();

    // Process paths
    for (path, path_item) in spec.paths.iter() {
        let path_item = match path_item {
            ReferenceOr::Reference { .. } => continue, // Skip references for simplicity
            ReferenceOr::Item(item) => item,
        };

        // Process operations (GET, POST, PUT, DELETE, etc.)
        process_operation(
            &mut tag_operations,
            &path,
            &path_item,
            &path_item.get,
            "get",
        );
        process_operation(
            &mut tag_operations,
            &path,
            &path_item,
            &path_item.post,
            "post",
        );
        process_operation(
            &mut tag_operations,
            &path,
            &path_item,
            &path_item.put,
            "put",
        );
        process_operation(
            &mut tag_operations,
            &path,
            &path_item,
            &path_item.delete,
            "delete",
        );
        process_operation(
            &mut tag_operations,
            &path,
            &path_item,
            &path_item.patch,
            "patch",
        );
    }

    // Convert grouped operations to commands
    for (tag, operations) in tag_operations {
        let mut tag_commands = HashMap::new();

        for (path, _path_item, operation) in operations {
            // Derive command name from operationId or path
            let command_name = if let Some(op_id) = &operation.operation_id {
                // Convert camelCase or PascalCase to kebab-case
                op_id.to_case(Case::Kebab)
            } else {
                // Use path as fallback, cleaned up
                let clean_path = path.replace('/', "-").trim_matches('-').to_string();
                clean_path
            };

            tag_commands.insert(
                command_name,
                Box::new(CommandType::Command(create_command_for_operation(
                    path,
                    &operation,
                    &spec.components,
                ))),
            );
        }

        if !tag_commands.is_empty() {
            // Use the tag name in kebab-case
            let tag_key = tag.to_case(Case::Kebab);
            config
                .commands
                .insert(tag_key, Box::new(CommandType::NestedCommand(tag_commands)));
        }
    }

    Ok(config)
}

fn process_operation<'a>(
    tag_operations: &mut HashMap<String, Vec<(&'a String, &'a PathItem, Operation)>>,
    path: &'a String,
    path_item: &'a PathItem,
    operation_opt: &'a Option<Operation>,
    _method: &str,
) {
    if let Some(operation) = operation_opt {
        // Get tag or use "default" if none specified
        let section = if let Some(tag) = operation.tags.first() {
            tag.clone()
        } else {
            let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            match segments.get(0) {
                Some(&segment) => segment.to_string(),
                None => "default".to_string(),
            }
        };

        tag_operations
            .entry(section)
            .or_insert_with(Vec::new)
            .push((path, path_item, operation.clone()));
    }
}

fn create_command_for_operation(
    path: &str,
    operation: &Operation,
    components: &Option<openapiv3::Components>,
) -> Command {
    let method: HttpMethod = match operation.operation_id.as_ref().map(|s| s.to_lowercase()) {
        Some(id) if id.starts_with("get") => HttpMethod::GET,
        Some(id) if id.starts_with("create") || id.starts_with("add") => HttpMethod::POST,
        Some(id) if id.starts_with("update") => HttpMethod::PUT,
        Some(id) if id.starts_with("delete") => HttpMethod::DELETE,
        Some(id) if id.starts_with("patch") => HttpMethod::PATCH,
        _ => {
            // Determine method from operation presence in PathItem
            // This is simplistic and based on the calling context
            if operation
                .operation_id
                .as_ref()
                .map_or(false, |id| id.starts_with("get"))
            {
                HttpMethod::GET
            } else if operation
                .operation_id
                .as_ref()
                .map_or(false, |id| id.starts_with("create"))
            {
                HttpMethod::POST
            } else if operation
                .operation_id
                .as_ref()
                .map_or(false, |id| id.starts_with("update"))
            {
                HttpMethod::PUT
            } else if operation
                .operation_id
                .as_ref()
                .map_or(false, |id| id.starts_with("delete"))
            {
                HttpMethod::DELETE
            } else {
                // Default to GET if unsure
                HttpMethod::GET
            }
        }
    };

    // Process path parameters
    let url = process_path_and_query(path, &operation.parameters);

    // Process request body if present
    let body = if let (Some(req_body), Some(components)) = (&operation.request_body, &components) {
        extract_request_body(req_body, components)
    } else {
        None
    };

    Command {
        method,
        url: format!("{{{{API_URL}}}}{}", url),
        body,
        postscript: None,
        headers: HashMap::new(),
    }
}

fn process_path_and_query(path: &str, parameters: &[ReferenceOr<Parameter>]) -> String {
    let mut result = path.to_string();
    let mut query_params = Vec::new();

    // Process path parameters - convert {param} to :param
    for param in parameters {
        if let ReferenceOr::Item(param_item) = param {
            if let Parameter::Path { parameter_data, .. } = param_item {
                let param_name = &parameter_data.name;
                result =
                    result.replace(&format!("{{{}}}", param_name), &format!(":{}", param_name));
            } else if let Parameter::Query { parameter_data, .. } = param_item {
                let param_name = &parameter_data.name;
                query_params.push(format!("{}=:{}", param_name, param_name));
            }
        }
    }

    // Add query parameters if any
    if !query_params.is_empty() {
        result = format!("{}?{}", result, query_params.join("&"));
    }

    result
}

fn extract_request_body(
    request_body: &ReferenceOr<RequestBody>,
    components: &openapiv3::Components,
) -> Option<Value> {
    match request_body {
        ReferenceOr::Item(body) => {
            // Try to get JSON schema
            if let Some(json_content) = body.content.get("application/json") {
                return extract_schema(&json_content.schema, components);
            }
            None
        }
        ReferenceOr::Reference { reference } => {
            // Handle reference to component
            let ref_parts: Vec<&str> = reference.split('/').collect();
            if ref_parts.len() == 4
                && ref_parts[1] == "components"
                && ref_parts[2] == "requestBodies"
            {
                let ref_name = ref_parts[3];
                if let Some(ref_body) = components.request_bodies.get(ref_name) {
                    return extract_request_body(ref_body, components);
                }
            }
            None
        }
    }
}

fn extract_schema(
    schema_opt: &Option<ReferenceOr<Schema>>,
    components: &openapiv3::Components,
) -> Option<Value> {
    if let Some(schema_ref) = schema_opt {
        match schema_ref {
            ReferenceOr::Item(schema) => process_schema(schema),
            ReferenceOr::Reference { reference } => {
                // Handle reference to component schema
                let ref_parts: Vec<&str> = reference.split('/').collect();
                if ref_parts.len() == 4 && ref_parts[1] == "components" && ref_parts[2] == "schemas"
                {
                    let ref_name = ref_parts[3];
                    if let Some(ref_schema) = components.schemas.get(ref_name) {
                        extract_schema(&Some(ref_schema.clone()), components)
                    } else {
                        // Return an empty object as placeholder if schema not found
                        Some(json!({}))
                    }
                } else {
                    None
                }
            }
        }
    } else {
        None
    }
}

fn process_schema(schema: &Schema) -> Option<Value> {
    match &schema.schema_kind {
        SchemaKind::Type(Type::Object(obj)) => {
            let mut properties = json!({});

            for (prop_name, prop_schema) in &obj.properties {
                let default_value = match &prop_schema {
                    ReferenceOr::Item(schema) => match &schema.schema_kind {
                        SchemaKind::Type(Type::String(_)) => json!(""),
                        SchemaKind::Type(Type::Number(_)) => json!(0),
                        SchemaKind::Type(Type::Integer(_)) => json!(0),
                        SchemaKind::Type(Type::Boolean {}) => json!(false),
                        SchemaKind::Type(Type::Array(_)) => json!([]),
                        SchemaKind::Type(Type::Object(_)) => json!({}),
                        _ => json!(null),
                    },
                    ReferenceOr::Reference { .. } => json!({}),
                };

                if let Some(obj) = properties.as_object_mut() {
                    obj.insert(prop_name.clone(), default_value);
                }
            }

            Some(properties)
        }
        _ => {
            // For non-object schemas, return null or a simple default
            Some(json!({}))
        }
    }
}

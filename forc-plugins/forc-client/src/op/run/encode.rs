use std::str::FromStr;

use fuel_abi_types::abi::full_program::{FullProgramABI, FullTypeApplication};
use fuels_core::{codec::ABIEncoder, types::unresolved_bytes::UnresolvedBytes};

const MAIN_KEYWORD: &str = "main";

// TODO: Move `Token` and and `Type` logic into a shared location in forc-client as they will
// likely to be re-used for contract and predicate calls as well.

/// A wrapper around fuels_core::types::Token, which enables serde de/serialization.
// TODO: implement serde::Serialize and serde::Deserialize. This is needed once we have the option
// to provide arguments in a file.
#[derive(Debug, PartialEq)]
struct Token(fuels_core::types::Token);

impl FromStr for Token {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let token = fuels_core::types::Token::from_str(s)?;
        Ok(Self(token))
    }
}

impl AsRef<fuels_core::types::Token> for Token {
    fn as_ref(&self) -> &fuels_core::types::Token {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Type {
    Unit,
    U8,
    U16,
    U32,
    U64,
    Bool,
    Byte,
    B256,
    Array,
    Vector,
    r#String,
    Struct,
    Enum,
    Tuple,
    RawSlice,
}

impl Token {
    /// Generate a new token using provided type information and the value for the argument.
    ///
    /// Generates an error if there is a mismatch between the type information and the provided
    /// value for that type.
    fn from_type_and_value(arg_type: &Type, value: &str) -> anyhow::Result<Self> {
        // TODO: Implement all types.
        // TODO: provide better error messages.
        match arg_type {
            Type::Unit => Ok(Token(fuels_core::types::Token::Unit)),
            Type::U8 => {
                let u8_val = value.parse::<u8>()?;
                Ok(Token(fuels_core::types::Token::U8(u8_val)))
            }
            Type::U16 => {
                let u16_val = value.parse::<u16>()?;
                Ok(Token(fuels_core::types::Token::U16(u16_val)))
            }
            Type::U32 => {
                let u32_val = value.parse::<u32>()?;
                Ok(Token(fuels_core::types::Token::U32(u32_val)))
            }
            Type::U64 => {
                let u64_val = value.parse::<u64>()?;
                Ok(Token(fuels_core::types::Token::U64(u64_val)))
            }
            Type::Bool => {
                let bool_val = value.parse::<bool>()?;
                Ok(Token(fuels_core::types::Token::Bool(bool_val)))
            }
            _ => anyhow::bail!("{arg_type:?} is not supported."),
        }
    }
}

impl FromStr for Type {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "()" => Ok(Type::Unit),
            "u8" => Ok(Type::U8),
            "u16" => Ok(Type::U16),
            "u32" => Ok(Type::U32),
            "u64" => Ok(Type::U64),
            "bool" => Ok(Type::Bool),
            other => anyhow::bail!("{other} type is not supported."),
        }
    }
}

impl TryFrom<&FullTypeApplication> for Type {
    type Error = anyhow::Error;

    fn try_from(value: &FullTypeApplication) -> Result<Self, Self::Error> {
        let type_field_string = &value.type_decl.type_field;
        Type::from_str(type_field_string)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ScriptCallHandler {
    main_arg_types: Vec<Type>,
}

impl ScriptCallHandler {
    /// Generate a new call handler for calling script main function from the json abi.
    ///
    /// Proviede json abi is used for determining the argument types, this is required as the data
    /// encoding is requires the type of the data.
    pub(crate) fn from_json_abi_str(json_abi_str: &str) -> anyhow::Result<Self> {
        let full_abi = FullProgramABI::from_json_abi(json_abi_str)?;
        // Note: using .expect() here is safe since a script without a main function is a compile
        // error and the fact that we have the json abi of the built script suggests that this is a
        // valid script.
        let main_function = full_abi
            .functions
            .iter()
            .find(|abi_func| abi_func.name() == MAIN_KEYWORD)
            .expect("every valid script needs to have a main function");
        let main_arg_types = main_function
            .inputs()
            .iter()
            .map(Type::try_from)
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(Self { main_arg_types })
    }

    /// Encode the provided values with script's main argument types.
    ///
    /// Returns an error if the provided value count does not match the number of arguments.
    pub(crate) fn encode_arguments(&self, values: &[&str]) -> anyhow::Result<UnresolvedBytes> {
        let main_arg_types = &self.main_arg_types;
        let expected_arg_count = main_arg_types.len();
        let provided_arg_count = values.len();

        if expected_arg_count != provided_arg_count {
            anyhow::bail!(
                "main function takes {expected_arg_count} arguments, {provided_arg_count} provided"
            );
        }

        let tokens = main_arg_types
            .iter()
            .zip(values.iter())
            .map(|(ty, val)| Token::from_type_and_value(ty, val).map(|token| token.0))
            .collect::<anyhow::Result<Vec<_>>>()?;

        Ok(ABIEncoder::encode(tokens.as_slice())?)
    }
}

#[cfg(test)]
mod tests {
    use super::{ScriptCallHandler, Token, Type};
    use std::str::FromStr;

    #[test]
    fn test_script_call_handler_generation_success() {
        let test_json_abi = r#"{"types":[{"typeId":0,"type":"()","components":[],"typeParameters":null},
{"typeId":1,"type":"bool","components":null,"typeParameters":null},{"typeId":2,"type":"u8","components":null,
"typeParameters":null}],"functions":[{"inputs":[{"name":"test_u8","type":2,"typeArguments":null},{"name":"test_bool",
"type":1,"typeArguments":null}],"name":"main","output":{"name":"","type":0,"typeArguments":null},"attributes":null}],
"loggedTypes":[],"messagesTypes":[],"configurables":[]}"#;
        let generated_call_handler = ScriptCallHandler::from_json_abi_str(test_json_abi).unwrap();

        let expected_call_handler = ScriptCallHandler {
            main_arg_types: vec![Type::U8, Type::Bool],
        };

        assert_eq!(generated_call_handler, expected_call_handler);
    }

    #[test]
    #[should_panic]
    fn test_script_call_handler_generation_fail_missing_main() {
        let test_json_abi =
            r#"{"types":[],"functions":[],"loggedTypes":[],"messagesTypes":[],"configurables":[]}"#;
        ScriptCallHandler::from_json_abi_str(test_json_abi).unwrap();
    }

    #[test]
    fn test_main_encoding_success() {
        let test_json_abi = r#"{"types":[{"typeId":0,"type":"()","components":[],"typeParameters":null},
{"typeId":1,"type":"bool","components":null,"typeParameters":null},{"typeId":2,"type":"u8","components":null,
"typeParameters":null}],"functions":[{"inputs":[{"name":"test_u8","type":2,"typeArguments":null},{"name":"test_bool",
"type":1,"typeArguments":null}],"name":"main","output":{"name":"","type":0,"typeArguments":null},"attributes":null}],
"loggedTypes":[],"messagesTypes":[],"configurables":[]}"#;
        let call_handler = ScriptCallHandler::from_json_abi_str(test_json_abi).unwrap();
        let values = ["2", "true"];

        let test_data_offset = 0;
        let encoded_bytes = call_handler
            .encode_arguments(&values)
            .unwrap()
            .resolve(test_data_offset);
        let expected_bytes = vec![
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8,
        ];
        assert_eq!(encoded_bytes, expected_bytes);
    }

    #[test]
    #[should_panic]
    fn test_main_encoding_fail_arg_type_mismatch() {
        let test_json_abi = r#"{"types":[{"typeId":0,"type":"()","components":[],"typeParameters":null},
{"typeId":1,"type":"bool","components":null,"typeParameters":null},{"typeId":2,"type":"u8","components":null,
"typeParameters":null}],"functions":[{"inputs":[{"name":"test_u8","type":2,"typeArguments":null},{"name":"test_bool",
"type":1,"typeArguments":null}],"name":"main","output":{"name":"","type":0,"typeArguments":null},"attributes":null}],
"loggedTypes":[],"messagesTypes":[],"configurables":[]}"#;
        let call_handler = ScriptCallHandler::from_json_abi_str(test_json_abi).unwrap();
        // The abi describes the following main function:
        // - fn main(test_u8: u8, test_bool: bool)
        // Providing a bool to u8 field should return an error.
        let values = ["true", "2"];
        call_handler.encode_arguments(&values).unwrap();
    }

    #[test]
    #[should_panic(expected = "main function takes 2 arguments, 1 provided")]
    fn test_main_encoding_fail_arg_count_mismatch() {
        let test_json_abi = r#"{"types":[{"typeId":0,"type":"()","components":[],"typeParameters":null},
{"typeId":1,"type":"bool","components":null,"typeParameters":null},{"typeId":2,"type":"u8","components":null,
"typeParameters":null}],"functions":[{"inputs":[{"name":"test_u8","type":2,"typeArguments":null},{"name":"test_bool",
"type":1,"typeArguments":null}],"name":"main","output":{"name":"","type":0,"typeArguments":null},"attributes":null}],
"loggedTypes":[],"messagesTypes":[],"configurables":[]}"#;
        let call_handler = ScriptCallHandler::from_json_abi_str(test_json_abi).unwrap();
        // The abi describes the following main function:
        // - fn main(test_u8: u8, test_bool: bool)
        // Providing only 1 value should return an error as function requires 2 args.
        let values = ["true"];
        call_handler.encode_arguments(&values).unwrap();
    }

    #[test]
    fn test_token_generation_success() {
        let u8_token = Token::from_type_and_value(&Type::U8, "1").unwrap();
        let u16_token = Token::from_type_and_value(&Type::U16, "1").unwrap();
        let u32_token = Token::from_type_and_value(&Type::U32, "1").unwrap();
        let u64_token = Token::from_type_and_value(&Type::U64, "1").unwrap();
        let bool_token = Token::from_type_and_value(&Type::Bool, "true").unwrap();

        let generated_tokens = [u8_token, u16_token, u32_token, u64_token, bool_token];
        let expected_tokens = [
            Token(fuels_core::types::Token::U8(1)),
            Token(fuels_core::types::Token::U16(1)),
            Token(fuels_core::types::Token::U32(1)),
            Token(fuels_core::types::Token::U64(1)),
            Token(fuels_core::types::Token::Bool(true)),
        ];

        assert_eq!(generated_tokens, expected_tokens)
    }

    #[test]
    #[should_panic]
    fn test_token_generation_fail_type_mismatch() {
        Token::from_type_and_value(&Type::U8, "false").unwrap();
    }

    #[test]
    fn test_type_generation_success() {
        let possible_type_list = ["()", "u8", "u16", "u32", "u64", "bool"];
        let types = possible_type_list
            .iter()
            .map(|type_str| Type::from_str(type_str))
            .collect::<anyhow::Result<Vec<_>>>()
            .unwrap();

        let expected_types = vec![
            Type::Unit,
            Type::U8,
            Type::U16,
            Type::U32,
            Type::U64,
            Type::Bool,
        ];
        assert_eq!(types, expected_types)
    }

    #[test]
    #[should_panic(expected = "u2 type is not supported.")]
    fn test_type_generation_fail_invalid_type() {
        let invalid_type_str = "u2";
        Type::from_str(invalid_type_str).unwrap();
    }
}

use crate::compiler::{as_module, compile_units};
use expect_test::expect;
use move_binary_format::errors::VMResult;
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{ModuleId, TypeTag},
    trace::CallTrace,
    value::MoveValue,
};
use move_vm_runtime::move_vm::MoveVM;
use move_vm_test_utils::InMemoryStorage;
use move_vm_types::gas::UnmeteredGasMeter;

const TEST_ADDR: AccountAddress = AccountAddress::new([42; AccountAddress::LENGTH]);
const TEST_MODULE_ID: &str = "M";

#[test]
fn call_traces_collection() {
    let code = code();
    let traces = run(
        &code,
        "test",
        vec![MoveValue::U64(42), MoveValue::Bool(true)],
        vec![TypeTag::U256],
    )
    .unwrap();

    expect![[r#"
        [
            CallTrace {
                depth: 1,
                call_type: CallGeneric,
                module_id: Some(
                    "0x2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a::M",
                ),
                function: "test",
                ty_args: [
                    U256,
                ],
                args: [
                    U64(
                        42,
                    ),
                    Bool(
                        true,
                    ),
                ],
                gas_used: 0,
                err: None,
            },
            CallTrace {
                depth: 2,
                call_type: Call,
                module_id: Some(
                    "0x2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a::M",
                ),
                function: "test2",
                ty_args: [],
                args: [
                    U64(
                        43,
                    ),
                    Bool(
                        true,
                    ),
                ],
                gas_used: 0,
                err: None,
            },
            CallTrace {
                depth: 3,
                call_type: Call,
                module_id: Some(
                    "0x2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a::M",
                ),
                function: "test3",
                ty_args: [],
                args: [
                    Struct(
                        WithTypes {
                            type_: StructTag {
                                address: 2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a,
                                module: Identifier(
                                    "M",
                                ),
                                name: Identifier(
                                    "Foo",
                                ),
                                type_params: [],
                            },
                            fields: [
                                (
                                    Identifier(
                                        "x",
                                    ),
                                    U64(
                                        43,
                                    ),
                                ),
                                (
                                    Identifier(
                                        "y",
                                    ),
                                    Bool(
                                        true,
                                    ),
                                ),
                            ],
                        },
                    ),
                ],
                gas_used: 0,
                err: None,
            },
        ]
    "#]]
    .assert_debug_eq(&traces);
}

fn code() -> ModuleCode {
    let code = format!(
        r#"
        module 0x{}::{} {{
            struct Foo has drop {{ x: u64, y: bool }}

            public fun test<T>(x: u64, y: bool) {{
                test2(x + 1, y);
            }}

            fun test2(x: u64, y: bool) {{
                let f = Foo {{ x, y }};

                test3(f);
            }}

            fun test3(f: Foo) {{
                let _ = f;
            }}
        }}
    "#,
        TEST_ADDR, TEST_MODULE_ID,
    );

    let module_id = ModuleId::new(TEST_ADDR, Identifier::new(TEST_MODULE_ID).unwrap());

    (module_id, code)
}

fn run(
    module: &ModuleCode,
    fun_name: &str,
    args: Vec<MoveValue>,
    ty_args: Vec<TypeTag>,
) -> VMResult<Vec<CallTrace>> {
    let module_id = &module.0;
    let modules = vec![module.clone()];
    let (vm, storage) = setup_vm(&modules);
    let mut session = vm.new_session(&storage);

    let fun_name = Identifier::new(fun_name).unwrap();

    let args: Vec<_> = args
        .into_iter()
        .map(|val| val.simple_serialize().unwrap())
        .collect();

    session
        .execute_function_bypass_visibility(
            module_id,
            &fun_name,
            ty_args,
            args,
            &mut UnmeteredGasMeter,
        )
        .map(|ret_values| ret_values.call_traces)
}

type ModuleCode = (ModuleId, String);

fn setup_vm(modules: &[ModuleCode]) -> (MoveVM, InMemoryStorage) {
    let mut storage = InMemoryStorage::new();
    compile_modules(&mut storage, modules);
    (MoveVM::new(vec![]).unwrap(), storage)
}

fn compile_modules(storage: &mut InMemoryStorage, modules: &[ModuleCode]) {
    modules.iter().for_each(|(id, code)| {
        compile_module(storage, id, code);
    });
}

fn compile_module(storage: &mut InMemoryStorage, mod_id: &ModuleId, code: &str) {
    let mut units = compile_units(code).unwrap();
    let module = as_module(units.pop().unwrap());
    let mut blob = vec![];
    module.serialize(&mut blob).unwrap();
    storage.publish_or_overwrite_module(mod_id.clone(), blob);
}

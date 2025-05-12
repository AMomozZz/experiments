use std::{cell::RefCell, rc::Rc, fmt::Debug};

use runtime::prelude::*;
use wasmtime::{component::{Component, Linker, TypedFunc}, Engine as WasmEngine, Store};
use wasmtime_wasi::{ResourceTable, WasiImpl};
use base64::{engine::general_purpose::STANDARD, Engine};
use runtime::prelude::serde::Deserialize;

// host
pub struct Host {
        ctx: wasmtime_wasi::WasiCtx,
        table: ResourceTable,
    }
    
    impl wasmtime_wasi::WasiView for Host {
        fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
            &mut self.ctx
        }
    }
    
    impl wasmtime_wasi::IoView for Host {
        fn table(&mut self) -> &mut ResourceTable {
            &mut self.table
        }
    }
    
    impl Host {
        pub fn new() -> Self {
            let ctx = wasmtime_wasi::WasiCtxBuilder::new().inherit_stdio().build();
            let table = ResourceTable::new();
            Self { ctx, table }
        }
    }

#[derive(Clone, Send, Sync, Timestamp)]
pub struct WasmFunction<I, O> {
    store: Rc<RefCell<Store<WasiImpl<Host>>>>,
    func: Option<TypedFunc<I, O>>,
    linker: Linker<WasiImpl<Host>>,
    engine: WasmEngine,
    pkg_name: Option<String>,
    name: Option<String>,
}

impl<I, O> Debug for WasmFunction<I, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmFunction")
        .field("func", &self.func.iter().len())
        .field("pkg_name", &self.pkg_name)
        .field("name", &self.name)
        .finish()
    }
}

impl<I, O> WasmFunction<I, O> 
where 
    I: wasmtime::component::Lower + wasmtime::component::ComponentNamedList,
    O: wasmtime::component::Lift + wasmtime::component::ComponentNamedList,
{
    pub fn new(linker: &Linker<WasiImpl<Host>>, engine: &WasmEngine, guest_wasi_module: &[u8], store_wrapper: &Rc<RefCell<Store<WasiImpl<Host>>>>, pkg_name: &str, name: &str) -> Self {
        // eprintln!("{}", guest_wasi_module.len()); // 92192
        let component = Component::from_binary(engine, guest_wasi_module).unwrap();
        // let a = component.serialize().unwrap();
        // eprintln!("{}", a.len()); // 341360
        let clone_store_wrapper = store_wrapper.clone();
        WasmFunction {
            func: Some(Self::_get_func_from_component(linker, &component, &clone_store_wrapper, pkg_name, name)),
            store: clone_store_wrapper,
            linker: linker.clone(),
            engine: engine.clone(),
            pkg_name: Some(pkg_name.to_string()),
            name: Some(name.to_string()),
        }
    }

    pub fn new_empty(linker: &Linker<WasiImpl<Host>>, engine: &WasmEngine, store_wrapper: &Rc<RefCell<Store<WasiImpl<Host>>>>) -> Self {
        let clone_store_wrapper = store_wrapper.clone();
        WasmFunction {
            func: None,
            store: clone_store_wrapper,
            linker: linker.clone(),
            engine: engine.clone(),
            pkg_name: None,
            name: None,
        }
    }

    pub fn new_empty_with_name(linker: &Linker<WasiImpl<Host>>, engine: &WasmEngine, store_wrapper: &Rc<RefCell<Store<WasiImpl<Host>>>>, pkg_name: &str, name: &str) -> Self {
        let clone_store_wrapper = store_wrapper.clone();
        WasmFunction {
            func: None,
            store: clone_store_wrapper,
            linker: linker.clone(),
            engine: engine.clone(),
            pkg_name: Some(pkg_name.to_string()),
            name: Some(name.to_string()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.func.is_none()
    }

    pub fn call(&self, input: I) -> O {
        match self.func {
            Some(f) => {
                let result = f.call(&mut *self.store.borrow_mut(), input).unwrap();
                f.post_return(&mut *self.store.borrow_mut()).unwrap();
                result
            },
            None => panic!("Function not found: {:?}", self),
        }
    }

    pub fn switch_default(&mut self, guest_wasi_module: &[u8]) {
        let default_pkg_name = match self.pkg_name {
            Some(ref pkg_name) => pkg_name.clone(),
            None => panic!("Default package name not found: {:?}", self),
        };
        let default_name = match self.name {
            Some(ref name) => name.clone(),
            None => panic!("Default function name not found: {:?}", self),
        };
        self.switch(guest_wasi_module, default_pkg_name.as_str(), default_name.as_str());
    }

    pub fn switch(&mut self, guest_wasi_module: &[u8], pkg_name: &str, name: &str) {
        let component = Component::from_binary(&self.engine, guest_wasi_module).unwrap();
        self.func = Some(Self::_get_func_from_component(&self.linker, &component, &self.store, pkg_name, name))
    }

    fn _get_func_from_component(linker: &Linker<WasiImpl<Host>>, component: &Component, store_wrapper: &Rc<RefCell<Store<WasiImpl<Host>>>>, pkg_name: &str, name: &str) -> wasmtime::component::TypedFunc<I, O> {
        let mut store = store_wrapper.borrow_mut();
        let instance = linker.instantiate(&mut *store, component).unwrap();
        let intf_export = instance
            .get_export(&mut *store, None, pkg_name)
            .unwrap();
        let func_export = instance
            .get_export(&mut *store, Some(&intf_export), name)
            .unwrap();
        instance
            .get_typed_func::<I, O>(&mut *store, func_export)
            .unwrap()
    }
}

#[derive(Debug, Clone, Send, DeepClone, serde::Serialize, serde::Deserialize, Timestamp, New)]
#[serde(crate = "runtime::prelude::serde")]
pub struct WasmComponent {
    #[serde(serialize_with = "serialize_vec_u8", deserialize_with = "deserialize_vec_u8")]
    pub file: Vec<u8>,
    pub pkg_name: String,
    pub name: String,
    #[timestamp]
    pub date_time: u64,
    pub extra: String,
}

fn serialize_vec_u8<S>(vec: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = STANDARD.encode(vec);
    serializer.serialize_str(&encoded)
}

fn deserialize_vec_u8<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let encoded: String = Deserialize::deserialize(deserializer)?;
    STANDARD.decode(&encoded).map_err(serde::de::Error::custom)
}

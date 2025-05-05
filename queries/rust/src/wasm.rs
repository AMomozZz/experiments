#![allow(unused_imports)]
use std::borrow::Cow;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;
use wasmtime::component::Accessor;
use wasmtime::component::AccessorTask;
use wasmtime::component::Component;
use wasmtime::component::ComponentExportIndex;
use wasmtime::component::ErrorContext;
use wasmtime::component::HostFuture;
use wasmtime::component::HostStream;
use wasmtime::component::Instance;
use wasmtime::component::Linker;
use wasmtime::component::ResourceTable;
use wasmtime::component::StreamReader;
use wasmtime::component::StreamWriter;
use wasmtime::component::TypedFunc;
use wasmtime::component::VecBuffer;
use wasmtime::CacheStore;
use wasmtime::Config;
use wasmtime::Engine as WasmEngine;
use wasmtime::Store;
use wasmtime_wasi::p3::cli::WasiCliCtx;
use wasmtime_wasi::p3::cli::WasiCliView;
use wasmtime_wasi::p3::clocks::WasiClocksCtx;
use wasmtime_wasi::p3::clocks::WasiClocksView;
use wasmtime_wasi::p3::filesystem::DirPerms;
use wasmtime_wasi::p3::filesystem::FilePerms;
use wasmtime_wasi::p3::filesystem::WasiFilesystemCtx;
use wasmtime_wasi::p3::filesystem::WasiFilesystemView;
use wasmtime_wasi::p3::random::WasiRandomCtx;
use wasmtime_wasi::p3::random::WasiRandomView;
use wasmtime_wasi::p3::sockets::AllowedNetworkUses;
use wasmtime_wasi::p3::sockets::SocketAddrCheck;
use wasmtime_wasi::p3::sockets::WasiSocketsCtx;
use wasmtime_wasi::p3::sockets::WasiSocketsView;
use wasmtime_wasi::p3::AccessorTaskFn;
use wasmtime_wasi::p3::ResourceView;
use wasmtime_wasi::p2::IoView;
use wasmtime_wasi::p2::WasiCtx;
use wasmtime_wasi::p2::WasiCtxBuilder;
use wasmtime_wasi::p2::WasiView;
use wasmtime_wasi::p2::WasiImpl;

use std::{cell::RefCell, rc::Rc, fmt::Debug};

use runtime::prelude::*;
// use wasmtime::{component::{Component, Linker, TypedFunc}, Engine as WasmEngine, Store};
// use wasmtime_wasi::{ResourceTable, WasiImpl};
use base64::{engine::general_purpose::STANDARD, Engine};
use runtime::prelude::serde::Deserialize;

// host
pub struct Host {
        ctx: WasiCtx,
        table: ResourceTable,
    }
    
    impl WasiView for Host {
        fn ctx(&mut self) -> &mut WasiCtx {
            &mut self.ctx
        }
    }
    
    impl IoView for Host {
        fn table(&mut self) -> &mut ResourceTable {
            &mut self.table
        }
    }
    
    impl Host {
        pub fn new() -> Self {
            let ctx = WasiCtxBuilder::new().inherit_stdio().build();
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
    I: wasmtime::component::Lower + wasmtime::component::ComponentNamedList + std::marker::Sync + std::marker::Send,
    O: wasmtime::component::Lift + wasmtime::component::ComponentNamedList + std::marker::Send + std::marker::Sync + 'static,
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

    pub async fn call_async(&self, input: I) -> O {
        enum Event {
            Write((Option<StreamWriter<VecBuffer<String>>>, VecBuffer<String>)),
            Read((Option<StreamReader<Vec<String>>>, Vec<String>)),
        }
    
        let mut set = PromisesUnordered::<Event>::new();
    
        let func3: TypedFunc<(HostStream<String>,), (HostStream<String>,)> =
            instance.get_typed_func(&mut *self.store, export).unwrap();
    
        let buf = Vec::with_capacity(1024);
        let (tx, rx) = instance
            .stream::<String, VecBuffer<String>, Vec<String>, _, _>(&mut *self.store)
            .unwrap();
    
        let (result,) = func3.call_async(&mut *self.store, (rx.into(),)).await.unwrap();

        let res = match self.func {
            Some(f) => {
                let result = f.call_async(&mut *self.store.borrow_mut(), input).await.unwrap();
                f.post_return(&mut *self.store.borrow_mut()).unwrap();
                result
            },
            None => panic!("Function not found: {:?}", self),
        };
    
        set.push(
            tx.write(VecBuffer::from(vec!["Hello World! (test4)".to_owned()]))
                .map(Event::Write),
        );
        set.push(result.into_reader(&mut *self.store).read(buf).map(Event::Read));
    
        func3.post_return_async(&mut *self.store).await.unwrap();
    
        while let Ok(Some(event)) = set.next(&mut *self.store).await {
            match event {
                Event::Write((Some(tx), _)) => {
                    println!("Writing");
                    set.push(
                        tx.write(VecBuffer::from(vec!["Hello World! (test4)".to_owned()]))
                            .map(Event::Write),
                    );
                }
                Event::Write(_) => {
                    println!("Write finished");
                }
                Event::Read((Some(reader), buf)) => {
                    println!("Reading: {:?}", buf);
                    set.push(reader.read(buf).map(Event::Read));
                }
                Event::Read(_) => {
                    println!("Read error");
                }
            }
        }
        println!("All done");
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
        let (_, intf_export) = instance
            .get_export(&mut *store, None, pkg_name)
            .unwrap();
        let (_, func_export) = instance
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

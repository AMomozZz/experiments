use std::{cell::RefCell, rc::Rc, fmt::Debug};

use runtime::prelude::*;
use wasmtime::{component::{Component, Linker, TypedFunc}, Engine, Store};
use wasmtime_wasi::{ResourceTable, WasiImpl};

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
    // component: &'a [u8],
    store: Rc<RefCell<Store<WasiImpl<Host>>>>,
    func: TypedFunc<I, O>,
    linker: Linker<WasiImpl<Host>>,
    engine: Engine,
    // #[timestamp]
    // date_time: u64,
}

impl<I, O> Debug for WasmFunction<I, O> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasmFunction").finish()
    }
}

impl<I, O> WasmFunction<I, O> 
where 
    I: wasmtime::component::Lower + wasmtime::component::ComponentNamedList,
    O: wasmtime::component::Lift + wasmtime::component::ComponentNamedList
{
    pub fn new(linker: &Linker<WasiImpl<Host>>, engine: &Engine, guest_wasi_module: &[u8], store_wrapper: &Rc<RefCell<Store<WasiImpl<Host>>>>, pkg_name: &str, name: &str) -> Self {
        let component = Component::from_binary(engine, guest_wasi_module).unwrap();
        let clone_store_wrapper = store_wrapper.clone();
        WasmFunction {
            func: Self::_get_func_from_component(linker, &component, &clone_store_wrapper, pkg_name, name),
            store: clone_store_wrapper,
            linker: linker.clone(),
            engine: engine.clone(),
            // component,
            // date_time: todo!(),
        }
    }
    pub fn call(&self, input: I) -> O {
        let result = self.func.call(&mut *self.store.borrow_mut(), input).unwrap();
        self.func.post_return(&mut *self.store.borrow_mut()).unwrap();
        result
    }

    pub fn switch(&mut self, guest_wasi_module: &[u8], pkg_name: &str, name: &str) {
        // self.func = Self::_get_func_from_component(&self.linker, new_component, &self.store, pkg_name, name);
        // let clone_engine = self.engine.clone();
        // let clone_linker = self.linker.clone();
        // let clone_store_wrapper = self.store.clone();

        let component = Component::from_binary(&self.engine, guest_wasi_module).unwrap();
        // WasmFunction {
        self.func = Self::_get_func_from_component(&self.linker, &component, &self.store, pkg_name, name)//,
        //     store: clone_store_wrapper,
        //     linker: clone_linker,
        //     engine: clone_engine,
        //     // component: new_component,
        //     // date_time: todo!(),
        // }
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
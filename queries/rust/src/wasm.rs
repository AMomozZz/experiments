use std::{cell::RefCell, rc::Rc, fmt::Debug};

use runtime::prelude::*;
use wasmtime::{component::{Component, Linker, TypedFunc}, Store};
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

// #[derive(Debug, Clone, Send, Timestamp, New)]
// pub struct Component<'a> {
//     component: &'a [u8],
//     #[timestamp]
//     date_time: u64,
// }

#[derive(Clone, Send, Sync, Timestamp)]
pub struct WasmFunction<I, O> {
    // component: &'a [u8],
    store: Rc<RefCell<Store<WasiImpl<Host>>>>,
    func: TypedFunc<I, O>,
    linker: Linker<WasiImpl<Host>>,
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
    pub fn new(linker: &Linker<WasiImpl<Host>>, component: &Component, store_wrapper: &Rc<RefCell<Store<WasiImpl<Host>>>>, pkg_name: &str, name: &str) -> Self {
        let clone_store_wrapper = store_wrapper.clone();
        WasmFunction {
            func: Self::_get_func_from_component(linker, component, &clone_store_wrapper, pkg_name, name),
            store: clone_store_wrapper,
            linker: linker.clone(),
            // component,
            // date_time: todo!(),
        }
    }
    pub fn call(&self, input: I) -> O {
        let result = self.func.call(&mut *self.store.borrow_mut(), input).unwrap();
        self.func.post_return(&mut *self.store.borrow_mut()).unwrap();
        result
    }

    pub fn switch(&self, new_component: &Component, pkg_name: &str, name: &str) -> Self {
        // self.func = Self::_get_func_from_component(&self.linker, new_component, &self.store, pkg_name, name);
        let clone_linker = self.linker.clone();
        let clone_store_wrapper = self.store.clone();
        WasmFunction {
            func: Self::_get_func_from_component(&clone_linker, new_component, &clone_store_wrapper, pkg_name, name),
            store: clone_store_wrapper,
            linker: clone_linker,
            // component: new_component,
            // date_time: todo!(),
        }
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
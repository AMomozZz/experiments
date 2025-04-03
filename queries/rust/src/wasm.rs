use std::{cell::RefCell, rc::Rc, fmt::Debug};

use runtime::prelude::{serde::{Deserialize, Serialize}, *};
use wasmtime::{component::{Component, Linker, TypedFunc}, Engine, Store};
use wasmtime_wasi::{ResourceTable, WasiImpl};
use stream::Event;

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
    pkg_name: String,
    name: String,
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
    I: wasmtime::component::Lower + wasmtime::component::ComponentNamedList + Clone + Send + Sync + Serialize + for<'a> Deserialize<'a> + Unpin + std::fmt::Debug + 'static,
    O: wasmtime::component::Lift + wasmtime::component::ComponentNamedList + Clone + Send + Sync + Serialize + for<'a> Deserialize<'a> + Unpin + std::fmt::Debug + 'static,
{
    pub fn new(linker: &Linker<WasiImpl<Host>>, engine: &Engine, guest_wasi_module: &[u8], store_wrapper: &Rc<RefCell<Store<WasiImpl<Host>>>>, pkg_name: &str, name: &str) -> Self {
        // eprintln!("{}", guest_wasi_module.len()); // 92192
        let component = Component::from_binary(engine, guest_wasi_module).unwrap();
        // let a = component.serialize().unwrap();
        // eprintln!("{}", a.len()); // 341360
        let clone_store_wrapper = store_wrapper.clone();
        WasmFunction {
            func: Self::_get_func_from_component(linker, &component, &clone_store_wrapper, pkg_name, name),
            store: clone_store_wrapper,
            linker: linker.clone(),
            engine: engine.clone(),
            pkg_name: pkg_name.to_string(),
            name: name.to_string(),
            // component,
            // date_time: todo!(),
        }
    }
    pub fn call(&self, input: I) -> O {
        let result = self.func.call(&mut *self.store.borrow_mut(), input).unwrap();
        self.func.post_return(&mut *self.store.borrow_mut()).unwrap();
        result
    }

    pub fn switch_default(&mut self, guest_wasi_module: &[u8]) {
        let default_pkg_name = self.pkg_name.clone();
        let default_name = self.name.clone();
        self.switch(guest_wasi_module, default_pkg_name.as_str(), default_name.as_str());
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

    pub fn run_wasm_operator(
        mut data: Stream<I>, 
        mut components: Stream<Vec<u8>>, 
        ctx: &mut Context,
        linker: &Linker<WasiImpl<Host>>, 
        engine: &Engine,
        store_wrapper: &Rc<RefCell<Store<WasiImpl<Host>>>>,
        pkg_name: &str,
        name: &str,
    ) {

        ctx.operator(move |tx: stream::Collector<O>| async move {
            // initialise WASM
            let mut func: Option<WasmFunction<I, O>> = None;
            tokio::select! {
                event = components.recv() => {
                    loop {
                        match event {
                            Event::Data(_time, data) => {
                                // update func
                                // if let Some(ref mut f) = func {
                                //     f.switch_default(&data);
                                // }
                                match func {
                                    Some(ref mut f) => f.switch_default(&data),
                                    None => {
                                        let clone_store_wrapper = store_wrapper.clone();
                                        func = Some(WasmFunction::new(linker, engine, &data, &clone_store_wrapper, pkg_name, name))
                                    },
                                }
                            },
                            Event::Watermark(time) => tx.send(Event::Watermark(time)).await?,
                            Event::Snapshot(id) => tx.send(Event::Snapshot(id)).await?,
                            Event::Sentinel => {
                                tx.send(Event::Sentinel).await?;
                                break;
                            },
                        }
                    }
                },
                event = data.recv() => {
                    loop {
                        match event {
                            Event::Data(time, data) => {
                                if let Some(ref mut f) = func {
                                    // Call the function with the data
                                    // f.call(data);
                                    tx.send(Event::Data(time, f.call(data))).await?
                                }
                            },
                            Event::Watermark(time) => tx.send(Event::Watermark(time)).await?,
                            Event::Snapshot(id) => tx.send(Event::Snapshot(id)).await?,
                            Event::Sentinel => {
                                tx.send(Event::Sentinel).await?;
                                break;
                            },
                        }
                    }
                    
                },
            }
            Ok(())
        })
        .drain(ctx);
    }
}
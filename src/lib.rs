#![allow(non_snake_case, non_camel_case_types)]

use std::sync::RwLock;

use windows::{core::{implement, IInspectable, ManuallyDrop, HSTRING, HRESULT, Result}, Win32::{System::WinRT::{IActivationFactory_Impl, IActivationFactory}, Foundation::S_OK}};

mod bindings;

#[implement(bindings::Class)]
struct Class(RwLock<i32>);

impl bindings::IClass_Impl for Class {
    fn Property(&self) -> Result<i32> {
        let reader = self.0.read().unwrap();
        Ok(*reader)
    }
    fn SetProperty(&self, value: i32) -> Result<()> {
        let mut writer = self.0.write().unwrap();
        *writer = value;
        Ok(())
    }
    fn Make(&self, value: i32) -> Result<bindings::Class> {
        Ok(Class(RwLock::new(value)).into())
    }
    fn MakeTypeErased(&self, value: i32) -> Result<IInspectable> {
        Ok(Class(RwLock::new(value)).into())
    }
}

#[implement(IActivationFactory, bindings::IClassFactory)]
struct ClassFactory();

impl IActivationFactory_Impl for ClassFactory {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        Ok(Class(RwLock::new(0)).into())
    }
}

impl bindings::IClassFactory_Impl for ClassFactory {
    fn CreateInstance(&self, property: i32) -> Result<bindings::Class> {
        Ok(Class(RwLock::new(property)).into())
    }
}

#[no_mangle]
unsafe extern "stdcall" fn DllGetActivationFactory(
    _name: ManuallyDrop<HSTRING>,
    result: *mut *mut std::ffi::c_void,
) -> HRESULT {
    // TODO: check class name
    let factory: IActivationFactory = ClassFactory().into();
    *result = std::mem::transmute(factory);
    S_OK
}

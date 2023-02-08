#![allow(non_snake_case, non_camel_case_types)]

use std::{sync::RwLock};

use bindings::IChewyStyleFactory;
use taffy::{Taffy, prelude::{Node, AvailableSpace, Size, Rect}, style::{Style, FlexDirection, FlexWrap, Dimension}, error::TaffyResult};
use windows::{core::{implement, IInspectable, ManuallyDrop, HSTRING, HRESULT, Result, PCWSTR, AsImpl}, Win32::{System::WinRT::{IActivationFactory_Impl, IActivationFactory}, Foundation::{S_OK, E_INVALIDARG, E_BOUNDS}}, w, Foundation::Point};

mod bindings;

const CHEWY_TAFFY_NAME: PCWSTR = w!("Chewy.ChewyTaffy");
const CHEWY_STYLE_NAME: PCWSTR = w!("Chewy.ChewyStyle");

#[implement(bindings::ChewyTaffy)]
struct ChewyTaffy(RwLock<Taffy>);

impl bindings::IChewyTaffy_Impl for ChewyTaffy {
    fn NewLeaf(
        &self,
        style: &core::option::Option<bindings::ChewyStyle>,
    ) -> windows::core::Result<bindings::ChewyNode> {
        let style = if let Some(style) = style.as_ref() {
            style
        } else {
            return Err(E_INVALIDARG.into());
        };
        let style = style.as_impl();
        let taffy_style = {
            style.0.read().unwrap().clone()
        };

        let mut taffy = self.0.write().unwrap();
        let node = taffy.new_leaf(taffy_style).win_ok()?;

        assert_eq!(std::mem::size_of::<bindings::ChewyNode>(), std::mem::size_of::<Node>());
        let chewy_node = unsafe { std::mem::transmute(node) };
        Ok(chewy_node)
    }

    fn SetChildren(
        &self,
        node: &bindings::ChewyNode,
        children: &core::option::Option<
            windows::Foundation::Collections::IVectorView<bindings::ChewyNode>,
        >,
    ) -> windows::core::Result<()> {
        assert_eq!(std::mem::size_of::<bindings::ChewyNode>(), std::mem::size_of::<Node>());

        let taffy_node: Node = unsafe { std::mem::transmute(*node) };
        let children = if let Some(children) = children.as_ref() {
            children
        } else {
            return Err(E_INVALIDARG.into());
        };

        let mut new_children = Vec::with_capacity(children.Size()? as usize);
        for child in children {
            let taffy_node: Node = unsafe { std::mem::transmute(child) };
            new_children.push(taffy_node);
        }

        let mut taffy = self.0.write().unwrap();
        taffy.set_children(taffy_node, &new_children).win_ok()?;

        Ok(())
    }

    fn ComputeLayout(
        &self,
        root_node: &bindings::ChewyNode,
        width: i32,
        height: i32,
    ) -> windows::core::Result<()> {
        assert_eq!(std::mem::size_of::<bindings::ChewyNode>(), std::mem::size_of::<Node>());
        let taffy_node: Node = unsafe { std::mem::transmute(*root_node) };

        let new_width = if width >= 0 {
            AvailableSpace::Definite(width as f32)
        } else {
            AvailableSpace::MaxContent
        };
        let new_height = if height >= 0 {
            AvailableSpace::Definite(height as f32)
        } else {
            AvailableSpace::MaxContent
        };

        let mut taffy = self.0.write().unwrap();
        taffy.compute_layout(taffy_node, Size { width: new_width, height: new_height }).win_ok()?;

        Ok(())
    }

    fn GetLayout(&self, node: &bindings::ChewyNode) -> windows::core::Result<bindings::ChewyLayout> {
        assert_eq!(std::mem::size_of::<bindings::ChewyNode>(), std::mem::size_of::<Node>());
        let taffy_node: Node = unsafe { std::mem::transmute(*node) };

        let taffy = self.0.read().unwrap();
        let taffy_layout = taffy.layout(taffy_node).win_ok()?;

        let layout = bindings::ChewyLayout {
            Order: taffy_layout.order,
            Size: windows::Foundation::Size { Width: taffy_layout.size.width, Height: taffy_layout.size.height },
            Location: Point { X: taffy_layout.location.x, Y: taffy_layout.location.y },
        };
        Ok(layout)
    }
}

#[implement(IActivationFactory)]
struct ChewyTaffyFactory();

impl IActivationFactory_Impl for ChewyTaffyFactory {
    fn ActivateInstance(&self) -> Result<IInspectable> {
        Ok(ChewyTaffy(RwLock::new(Taffy::new())).into())
    }
}

#[implement(bindings::ChewyStyle)]
struct ChewyStyle(RwLock<Style>);

impl bindings::IChewyStyle_Impl for ChewyStyle {}

#[implement(bindings::IChewyStyleFactory)]
struct ChewyStyleFactory();

impl bindings::IChewyStyleFactory_Impl for ChewyStyleFactory {
    fn CreateInstance(
        &self,
        style: &windows::core::HSTRING,
    ) -> windows::core::Result<bindings::ChewyStyle> {
        let style_string = style.to_string();
        let pairs = style_string.split(';');

        let mut taffy_style = Style::default();
        for pair in pairs {
            if let Some((property_name, property_value)) = pair.split_once(':') {
                let property_value = property_value.trim();
                match property_name {
                    "flex-direction" => {
                        let direction = match property_value {
                            "row" => FlexDirection::Row,
                            "column" => FlexDirection::Column,
                            "row-reverse" => FlexDirection::RowReverse,
                            "column-reverse" => FlexDirection::ColumnReverse,
                            _ => {
                                return Err(E_INVALIDARG.into());
                            }
                        };
                        taffy_style.flex_direction = direction;
                    },
                    "flex-wrap" => {
                        let wrap = match property_value {
                            "nowrap" => FlexWrap::NoWrap,
                            "wrap" => FlexWrap::Wrap,
                            "wrap-reverse" => FlexWrap::WrapReverse,
                            _ => {
                                return Err(E_INVALIDARG.into());
                            }
                        };
                        taffy_style.flex_wrap = wrap;
                    },
                    "width" => {
                        taffy_style.size.width = parse_dimension(property_value)?;
                    },
                    "height" => {
                        taffy_style.size.height = parse_dimension(property_value)?;
                    },
                    "margin" => {
                        let value = parse_dimension(property_value)?;
                        taffy_style.margin = Rect {
                            left: value,
                            top: value,
                            right: value,
                            bottom: value
                        };
                    },
                    _ => {
                        return Err(E_INVALIDARG.into());
                    }
                }
            } else {
                return Err(E_INVALIDARG.into());
            }
        }

        Ok(ChewyStyle(RwLock::new(taffy_style)).into())
    }
}

fn parse_dimension(property_value: &str) -> Result<Dimension> {
    let value = if property_value == "auto" {
        Dimension::Auto
    } else if property_value.ends_with('%') {
        let value = parse_f32(&property_value[0..property_value.len() - 1])?;
        Dimension::Percent(value)
    } else if property_value.ends_with("px") {
        let value = parse_f32(&property_value[0..property_value.len() - 2])?;
        Dimension::Points(value)
    } else {
        return Err(E_INVALIDARG.into());
    };
    Ok(value)
}

#[no_mangle]
unsafe extern "stdcall" fn DllGetActivationFactory(
    name: ManuallyDrop<HSTRING>,
    result: *mut *mut std::ffi::c_void,
) -> HRESULT {
    let name: PCWSTR = if let Some(name) = name.as_ref() {
        PCWSTR(name.as_ptr())
    } else {
        return E_INVALIDARG;
    };

    let factory: *mut std::ffi::c_void = match name {
        CHEWY_TAFFY_NAME => {
            std::mem::transmute::<IActivationFactory, _>(ChewyTaffyFactory().into())
        },
        CHEWY_STYLE_NAME => {
            std::mem::transmute::<IChewyStyleFactory, _>(ChewyStyleFactory().into())
        }
        _ => {
            return E_INVALIDARG;
        }
    };

    *result = std::mem::transmute(factory);
    S_OK
}

trait ToWindowsResult<T> {
    fn win_ok(self) -> Result<T>;
}

impl<T> ToWindowsResult<T> for TaffyResult<T> {
    fn win_ok(self) -> Result<T> {
        match self {
            Ok(result) => Ok(result),
            Err(taffy_error) => {
                let error = match taffy_error {
                    taffy::error::TaffyError::ChildIndexOutOfBounds { parent: _, child_index: _, child_count: _ } => E_BOUNDS,
                    taffy::error::TaffyError::InvalidParentNode(_) => E_INVALIDARG,
                    taffy::error::TaffyError::InvalidChildNode(_) => E_INVALIDARG,
                    taffy::error::TaffyError::InvalidInputNode(_) => E_INVALIDARG,
                };
                Err(error.into())
            },
        }
    }
}

fn parse_f32(str: &str) -> Result<f32> {
    let result = str.parse::<f32>();
    match result {
        Ok(value) => Ok(value),
        Err(_) => Err(E_INVALIDARG.into()),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn node_size_test() {
        assert_eq!(std::mem::size_of::<crate::bindings::ChewyNode>(), std::mem::size_of::<taffy::prelude::Node>());
    }
}
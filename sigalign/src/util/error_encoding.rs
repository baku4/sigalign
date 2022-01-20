use std::fmt::Debug;
use core::result::Result as CoreResult;
use anyhow::Result as AnyhowResult;
use anyhow::anyhow;

#[inline(always)]
pub fn transform_res_type<T, CE>(core_result: CoreResult<T, CE>) -> AnyhowResult<T> where
    CE: Debug
{
    match core_result {
        Ok(v) => Ok(v),
        Err(err) => Err(anyhow!("{:?}", err))
    }
}
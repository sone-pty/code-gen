use crate::error::Error;

use super::{InnerBuildContext, Template};

pub(crate) fn generate<W: std::io::Write + ?Sized>(
    template: &Template<'_>,
    stream: &mut W,
    tab_nums: i32,
    ctx: &InnerBuildContext<'_>,
    is_server: bool,
) -> Result<(), Error> {
    if is_server {
        inner_build_server(template, stream, tab_nums, ctx)?;
    } else {
        inner_build_client(template, stream, tab_nums, ctx)?;
    }
    Ok(())
}

pub(crate) fn inner_build_client<W: std::io::Write + ?Sized>(
    template: &Template<'_>,
    stream: &mut W,
    tab_nums: i32,
    ctx: &InnerBuildContext<'_>,
) -> Result<(), Error> {
    Ok(())
}

pub(crate) fn inner_build_server<W: std::io::Write + ?Sized>(
    template: &Template<'_>,
    stream: &mut W,
    tab_nums: i32,
    ctx: &InnerBuildContext<'_>,
) -> Result<(), Error> {
    Ok(())
}

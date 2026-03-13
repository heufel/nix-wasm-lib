use export_nix::export_nix;
use nix_types::NixValue;

#[export_nix]
pub fn test_export(arg1: NixValue, arg2: NixValue) -> NixValue {
    nix_types::NixValue::Int(1)
}

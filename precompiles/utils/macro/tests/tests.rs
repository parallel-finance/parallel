// Copyright 2021 Parallel Finance Developer.
// This file is part of Parallel Finance.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use sha3::{Digest, Keccak256};

#[precompile_utils_macro::generate_function_selector]
pub enum Action {
    Toto = "toto()",
    Tata = "tata()",
}

#[test]
fn test_keccak256() {
    assert_eq!(
        &precompile_utils_macro::keccak256!(""),
        Keccak256::digest(b"").as_slice(),
    );
    assert_eq!(
        &precompile_utils_macro::keccak256!("toto()"),
        Keccak256::digest(b"toto()").as_slice(),
    );
    assert_ne!(
        &precompile_utils_macro::keccak256!("toto()"),
        Keccak256::digest(b"tata()").as_slice(),
    );
}

#[test]
fn test_generate_function_selector() {
    assert_eq!(
        &(Action::Toto as u32).to_be_bytes()[..],
        &Keccak256::digest(b"toto()")[0..4],
    );
    assert_eq!(
        &(Action::Tata as u32).to_be_bytes()[..],
        &Keccak256::digest(b"tata()")[0..4],
    );
    assert_ne!(Action::Toto as u32, Action::Tata as u32);
}

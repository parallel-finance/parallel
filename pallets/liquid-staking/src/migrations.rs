// Copyright 2021-2022 Parallel Finance Developer.
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

use super::*;

pub mod v3 {
    use super::*;
    use crate::{Config, StorageVersion};
    use frame_support::{log, pallet_prelude::*, traits::Get};

    #[cfg(feature = "try-runtime")]
    pub fn pre_migrate<T: Config>() -> Result<(), &'static str> {
        Ok(())
    }

    pub fn migrate<T: Config>() -> Weight {
        0
    }

    #[cfg(feature = "try-runtime")]
    pub fn post_migrate<T: Config>() -> Result<(), &'static str> {
        Ok(())
    }
}

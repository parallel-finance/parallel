use frame_support::dispatch::Weight;

pub trait WeightInfo {
    fn trade() -> Weight;
}

impl WeightInfo for () {
    fn trade() -> Weight {
        1000u64.into()
    }
}

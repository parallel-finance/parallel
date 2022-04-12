pub trait EmergencyCallFilter<Call> {
    fn contains(call: &Call) -> bool;
}

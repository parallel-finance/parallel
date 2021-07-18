use parallel_primitives::Balance;

// Wrapper around the result of `Pallet::calculate_y`
pub struct YEvaluation {
    pub y: Balance,
    pub y_diff: Balance,
}

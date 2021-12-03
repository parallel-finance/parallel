use codec::{Decode, Encode};
use frame_support::{
    log,
    storage::{unhashed, StoragePrefixedMap},
    traits::{Currency, Get},
    weights::Weight,
    BoundedVec,
};
use sp_runtime::traits::CheckedDiv;
pub(crate) type BalanceOf<T> = <<T as orml_vesting::Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::Balance;
pub(crate) type VestingScheduleOf<T> =
    orml_vesting::VestingSchedule<<T as frame_system::Config>::BlockNumber, BalanceOf<T>>;

const OLD_START: u32 = 231619;
const OLD_PERIOD_COUNT: u32 = 2628000;

const NEW_START: u32 = 0;
const NEW_PERIOD: u32 = 1;
const NEW_PERIOD_COUNT: u32 = 2160000;
// total_update_schedules include 10731 items, plan to do two times' runtime upgrade
const MIGRATION_LIMIT: u32 = 5500;

pub fn migrate<T: orml_vesting::Config>() -> Weight {
    translate_values::<T, BoundedVec<VestingScheduleOf<T>, T::MaxVestingSchedules>, _>(|v| {
        let mut new_v = BoundedVec::default();
        v.iter().for_each(|vesting_schedule| {
            update_schedule::<T>(vesting_schedule)
                .and_then(|new_schedule| new_v.try_push(new_schedule).ok());
        });
        if !new_v.is_empty() && !new_v.len().eq(&v.len()) {
            log::warn!(target: "runtime::orml_vesting", "new schedule is not equal to old");
            return None;
        }
        if !new_v.is_empty() {
            return Some(new_v);
        }
        None
    });
    <T as frame_system::Config>::BlockWeights::get().max_block
}

// https://github.com/paritytech/substrate/blob/polkadot-v0.9.12/frame/support/src/storage/mod.rs#L1168-L1188
fn translate_values<
    T: orml_vesting::Config,
    Value: Decode + Encode,
    F: FnMut(Value) -> Option<Value>,
>(
    mut f: F,
) {
    log::info!(target: "runtime::orml_vesting", "migrate orml_vesting schedule");
    let mut count_write = 0u32;
    let mut count_read = 0u32;
    let prefix = orml_vesting::VestingSchedules::<T>::final_prefix();
    let mut previous_key = prefix.clone().to_vec();
    while let Some(next) =
        sp_io::storage::next_key(&previous_key).filter(|n| n.starts_with(&prefix))
    {
        if count_write.eq(&MIGRATION_LIMIT) {
            // Avoid terminate block production, so migrate the two within two runtime upgrade
            // refer to https://github.com/paritytech/substrate/issues/10407
            break;
        }
        count_read += 1;
        previous_key = next;
        let maybe_value = unhashed::get::<Value>(&previous_key);
        match maybe_value {
            Some(value) => match f(value) {
                Some(new) => {
                    unhashed::put::<Value>(&previous_key, &new);
                    count_write += 1;
                }
                None => continue,
            },
            None => {
                log::error!("old key failed to decode at {:?}", previous_key);
                continue;
            }
        }
    }
    log::info!(
        target: "runtime::orml_vesting",
        "count_read: {}, count_write: {}",
        count_read, count_write
    );
}

fn update_schedule<T: orml_vesting::Config>(
    vesting_schedule: &VestingScheduleOf<T>,
) -> Option<VestingScheduleOf<T>> {
    if !vesting_schedule.start.eq(&OLD_START.into())
        || !vesting_schedule.period_count.eq(&OLD_PERIOD_COUNT)
    {
        return None;
    }

    vesting_schedule
        .total_amount()
        .and_then(|total| total.checked_div(&NEW_PERIOD_COUNT.into()))
        .and_then(|per_period| {
            Some(VestingScheduleOf::<T> {
                start: NEW_START.into(),
                period: NEW_PERIOD.into(),
                period_count: NEW_PERIOD_COUNT,
                per_period,
            })
        })
}

/// Some checks prior to migration. This can be linked to
/// [`frame_support::traits::OnRuntimeUpgrade::pre_upgrade`] for further testing.
///
/// Panics if anything goes wrong.
#[cfg(feature = "try-runtime")]
pub fn pre_migrate<T: frame_system::Config + orml_vesting::Config>()
where
    u128: From<BalanceOf<T>>,
{
    let mut count_total = 0u64;
    let mut count_one = 0u64;
    let mut count_two = 0u64;
    let mut count_more = 0u64;
    let mut count_need_update = 0u64;
    let mut total_amount: BalanceOf<T> = 0u32.into();
    orml_vesting::VestingSchedules::<T>::iter().for_each(|(_k, v)| {
        count_total += 1;
        let length = v.len();
        if length == 1 {
            count_one += 1;
        } else if length == 2 {
            count_two += 1;
        } else if length > 2 {
            count_more += 1;
        }
        v.iter().for_each(|s| {
            if s.start.eq(&OLD_START.into()) && s.period_count.eq(&OLD_PERIOD_COUNT) {
                count_need_update += 1;
            }
            total_amount += s.per_period * s.period_count.into();
        });
    });

    // total accounts: 10680, one schedule: 10628, two schedule: 52, more schedule: 0, schedule need update: 10731, total_amount: 31450977794836396000
    log::info!(
        target: "runtime::orml_vesting",
        "{}, total accounts: {}, one schedule: {}, two schedule: {}, more schedule: {}, schedule need update: {}, total_amount: {:?}",
        "pre-migration", count_total, count_one, count_two, count_more, count_need_update,total_amount
    );
}

/// Some checks for after migration. This can be linked to
/// [`frame_support::traits::OnRuntimeUpgrade::post_upgrade`] for further testing.
///
/// Panics if anything goes wrong.
#[cfg(feature = "try-runtime")]
pub fn post_migrate<T: frame_system::Config + orml_vesting::Config>()
where
    u128: From<BalanceOf<T>>,
{
    let mut count_total = 0u64;
    let mut count_one = 0u64;
    let mut count_two = 0u64;
    let mut count_more = 0u64;
    let mut count_success_update = 0u64;
    let mut total_amount: BalanceOf<T> = 0u32.into();
    orml_vesting::VestingSchedules::<T>::iter().for_each(|(_k, v)| {
        count_total += 1;
        let length = v.len();
        if length == 1 {
            count_one += 1;
        } else if length == 2 {
            count_two += 1;
        } else if length > 2 {
            count_more += 1;
        }
        v.iter().for_each(|s| {
            if s.start.eq(&NEW_START.into()) && s.period_count.eq(&NEW_PERIOD_COUNT) {
                count_success_update += 1;
            }
            total_amount += s.per_period * s.period_count.into();
        });
    });

    // total accounts: 10680, one schedule: 10628, two schedule: 52, more schedule: 0, schedule success update: 10731, total_amount: 31450977784297360000
    log::info!(
        target: "runtime::orml_vesting",
        "{}, total accounts: {}, one schedule: {}, two schedule: {}, more schedule: {}, schedule success update: {}, total_amount: {:?}",
        "post-migration", count_total, count_one, count_two, count_more, count_success_update, total_amount
    );
}

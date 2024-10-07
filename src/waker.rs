use core::{
    mem::ManuallyDrop,
    ptr::{self, NonNull},
    task::{self, RawWaker, RawWakerVTable, Waker},
};

#[derive(Clone, Copy)]
pub struct StreamFrame {
    pub address: usize,
    pub out_ref: NonNull<()>,
    pub prev: NonNull<Option<StreamFrame>>,
}

struct WakerData<'a> {
    inner_waker: &'a Waker,
    frame: StreamFrame,
}

// -- VTable stuff start --

static WAKER_VTABLE: RawWakerVTable =
    RawWakerVTable::new(waker_clone, waker_wake, waker_wake_by_ref, waker_drop);

#[allow(unsafe_code)]
unsafe fn waker_clone(data: *const ()) -> RawWaker {
    // we are inside the waker vtable context and the void pointer points to our data struct
    let inner_waker = unsafe { &*data.cast::<WakerData<'_>>() }
        .inner_waker
        .clone();
    let inner_waker = ManuallyDrop::new(inner_waker);

    RawWaker::new(inner_waker.data(), inner_waker.vtable())
}

fn waker_wake(_data: *const ()) {
    unreachable!();
}

#[allow(unsafe_code)]
unsafe fn waker_wake_by_ref(data: *const ()) {
    // we are inside the waker vtable context and the void pointer points to our data struct
    let inner_waker = unsafe { &*data.cast::<WakerData<'_>>() }.inner_waker;
    inner_waker.wake_by_ref();
}

fn waker_drop(_data: *const ()) {
    unreachable!();
}

// -- VTable stuff end --

#[inline]
fn get_waker_data(waker: &Waker) -> Option<&WakerData<'_>> {
    if *waker.vtable() != WAKER_VTABLE {
        return None;
    }

    // we never set the data to null or a dangling pointer.
    #[allow(unsafe_code)]
    let data = unsafe {
        waker
            .data()
            .cast::<WakerData<'_>>()
            .as_ref()
            .unwrap_unchecked()
    };

    Some(data)
}

#[inline]
pub fn unwrap_inner(waker: &Waker) -> Option<&Waker> {
    let data = get_waker_data(waker)?;
    Some(data.inner_waker)
}

#[inline]
pub fn find_frame(waker: &Waker) -> Option<StreamFrame> {
    let data = get_waker_data(waker)?;
    Some(data.frame)
}

#[inline]
pub fn with_context<Item, F, Output>(
    waker: &Waker,
    stream_address: usize,
    out_ref: &mut Option<Item>,
    func: F,
) -> Output
where
    F: FnOnce(&mut task::Context<'_>) -> Output,
{
    let mut prev = find_frame(waker);

    // we construct the pointers from valid references. so they definitely aren't null.
    #[allow(unsafe_code)]
    let data = unsafe {
        WakerData {
            inner_waker: waker,
            frame: StreamFrame {
                address: stream_address,
                out_ref: NonNull::new_unchecked(ptr::from_mut(out_ref).cast()),
                prev: NonNull::new_unchecked(ptr::from_mut(&mut prev)),
            },
        }
    };

    // we only panic or proxy out to the inner waker.
    #[allow(unsafe_code)]
    let waker = unsafe { Waker::new(ptr::from_ref(&data).cast(), &WAKER_VTABLE) };
    let waker = ManuallyDrop::new(waker);

    let mut context = task::Context::from_waker(&waker);
    func(&mut context)
}

use crate::{
    error::try_map_opus_error,
    ffi,
    packet::{MutPacket, Packet},
    Result,
};

/// Returns Opus' internal `OpusRepacketizer`'s size in bytes.
pub fn repacketizer_size() -> usize {
    unsafe { ffi::opus_repacketizer_get_size() as usize }
}

pub fn multistream_packet_pad(
    mut data: MutPacket<'_>,
    new_len: usize,
    nb_streams: usize,
) -> Result<()> {
    try_map_opus_error(unsafe {
        ffi::opus_multistream_packet_pad(
            data.as_mut_ptr(),
            data.i32_len()?,
            new_len as i32,
            nb_streams as i32,
        )
    })
    .map(|_| ())
}

pub fn multistream_packet_unpad(mut data: MutPacket<'_>, nb_streams: usize) -> Result<()> {
    try_map_opus_error(unsafe {
        ffi::opus_multistream_packet_unpad(data.as_mut_ptr(), data.i32_len()?, nb_streams as i32)
    })
    .map(|_| ())
}

pub fn packet_pad(mut data: MutPacket<'_>, new_len: i32) -> Result<()> {
    try_map_opus_error(unsafe { ffi::opus_packet_pad(data.as_mut_ptr(), data.i32_len()?, new_len) })
        .map(|_| ())
}

pub fn packet_unpad(mut data: MutPacket<'_>) -> Result<()> {
    try_map_opus_error(unsafe { ffi::opus_packet_unpad(data.as_mut_ptr(), data.i32_len()?) })
        .map(|_| ())
}

#[derive(Debug)]
pub struct Repacketizer {
    pointer: *mut ffi::OpusRepacketizer,
}

impl Default for Repacketizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Repacketizer {
    /// We have to ensure that the resource our wrapping Opus-struct is pointing
    /// to is deallocated properly.
    fn drop(&mut self) {
        unsafe { ffi::opus_repacketizer_destroy(self.pointer) }
    }
}

impl Repacketizer {
    pub fn new() -> Self {
        let pointer = unsafe { ffi::opus_repacketizer_create() };

        Self { pointer }
    }

    pub fn nb_frames(&self) -> usize {
        unsafe { ffi::opus_repacketizer_get_nb_frames(self.pointer) as usize }
    }

    pub fn repacketizer_out(&self, mut data_out: MutPacket<'_>, max_len: i32) -> Result<()> {
        try_map_opus_error(unsafe {
            ffi::opus_repacketizer_out(self.pointer, data_out.as_mut_ptr(), max_len)
        })
        .map(|_| ())
    }

    pub fn repacketizer_out_range(
        &self,
        begin: i32,
        end: i32,
        mut data_out: MutPacket<'_>,
        max_len: i32,
    ) -> Result<()> {
        try_map_opus_error(unsafe {
            ffi::opus_repacketizer_out_range(
                self.pointer,
                begin,
                end,
                data_out.as_mut_ptr(),
                max_len,
            )
        })
        .map(|_| ())
    }

    pub fn repacketizer_cat(&self, data: Packet<'_>) -> Result<()> {
        try_map_opus_error(unsafe {
            ffi::opus_repacketizer_cat(self.pointer, data.as_ptr(), data.i32_len())
        })
        .map(|_| ())
    }
}

use crate::{Error, Result, TryInto, error::try_map_opus_error, ffi, packet::{MutPacket, Packet}};

/// Returns Opus' internal `OpusRepacketizer`'s size in bytes.
pub fn repacketizer_size() -> usize {
    unsafe { ffi::opus_repacketizer_get_size() as usize }
}

pub fn multistream_packet_pad<'a, TP>(data: TP, new_len: usize, nb_streams: usize) -> Result<()>
where
    TP: TryInto<MutPacket<'a>, Error = Error>,
{
    let mut data = data.try_into()?;

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

pub fn multistream_packet_unpad<'a, TP>(data: TP, nb_streams: usize) -> Result<()>
where
    TP: TryInto<MutPacket<'a>, Error = Error>,
{
    let mut data = data.try_into()?;

    try_map_opus_error(unsafe {
        ffi::opus_multistream_packet_unpad(data.as_mut_ptr(), data.i32_len()?, nb_streams as i32)
    })
    .map(|_| ())
}

pub fn packet_pad<'a, TP>(data: TP, new_len: i32) -> Result<()>
where
    TP: TryInto<MutPacket<'a>, Error = Error>,
{
    let mut data = data.try_into()?;

    try_map_opus_error(unsafe { ffi::opus_packet_pad(data.as_mut_ptr(), data.i32_len()?, new_len) })
        .map(|_| ())
}

pub fn packet_unpad<'a, TP>(data: TP) -> Result<()>
where
    TP: TryInto<MutPacket<'a>, Error = Error>,
{
    let mut data = data.try_into()?;

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

    pub fn repacketizer_out<'a, TP>(&self, data_out: TP, max_len: i32) -> Result<()>
    where
        TP: TryInto<MutPacket<'a>, Error = Error>,
    {
        let mut data_out = data_out.try_into()?;

        try_map_opus_error(unsafe {
            ffi::opus_repacketizer_out(self.pointer, data_out.as_mut_ptr(), max_len)
        })
        .map(|_| ())
    }

    pub fn repacketizer_out_range<'a, TP>(
        &self,
        begin: i32,
        end: i32,
        data_out: TP,
        max_len: i32,
    ) -> Result<()>
    where
        TP: TryInto<MutPacket<'a>, Error = Error>,
    {
        let mut data_out = data_out.try_into()?;

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

    pub fn repacketizer_cat<'a, TP>(&self, data: TP) -> Result<()>
    where
        TP: TryInto<Packet<'a>, Error = Error>,
    {
        let data = data.try_into()?;

        try_map_opus_error(unsafe {
            ffi::opus_repacketizer_cat(self.pointer, data.as_ptr(), data.i32_len())
        })
        .map(|_| ())
    }
}

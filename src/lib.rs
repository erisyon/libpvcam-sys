pub mod pvcam {
    mod internal {
        #![allow(non_upper_case_globals)]
        #![allow(non_camel_case_types)]
        #![allow(non_snake_case)]
        #![allow(dead_code)]
        include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

        extern "C" {
            // blacklisted from bindgen because it incorrectly identifies camera_name as *mut std::os::raw::c_char
            pub fn pl_cam_open(
                camera_name: *const std::os::raw::c_char,
                hcam: *mut i16,
                o_mode: i16,
            ) -> rs_bool;
        }
    }

    use std::ffi;
    use std::fmt;
    use std::os::raw as c_types;

    pub type Result<T> = std::result::Result<T, Error>;

    // TODO: work out how to make this
    #[derive(Debug, Clone)]
    pub struct Error {
        pub code: i16,
        pub message: String,
    }

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} (code: {})", self.message, self.code)
        }
    }

    // Implementation of this trait allows us to use the `?` form when
    // converting CStrings to pvcam::Result types; without it
    // the CString is not compatible with pvcam::Error
    impl std::convert::From<ffi::IntoStringError> for Error {
        fn from(error: ffi::IntoStringError) -> Self {
            Error {
                code: -1,
                message: format!("{:?} caused error", error.into_cstring()),
            }
        }
    }

    impl std::convert::From<Error> for std::string::String {
        fn from(e: Error) -> Self {
            e.message
        }
    }

    enum PVResult {
        Ok,
        Err,
    }

    fn check_call(res: c_types::c_ushort) -> PVResult {
        if (res as c_types::c_uint) == self::internal::PV_OK {
            return PVResult::Ok;
        }

        PVResult::Err
    }

    fn pvcam_error() -> Error {
        let code = unsafe { self::internal::pl_error_code() };
        let message = unsafe {
            let buf =
                ffi::CString::from_vec_unchecked(vec![0; self::internal::ERROR_MSG_LEN as usize])
                    .into_raw();

            match check_call(self::internal::pl_error_message(code, buf)) {
                // Cannot make use of the `?` form here because this function does not return a Result<T>
                PVResult::Ok => match ffi::CString::from_raw(buf).into_string() {
                    Ok(v) => v,
                    Err(e) => format!("Error converting pl_error_message to String: {}", e),
                },
                PVResult::Err => String::from("Unknown Error"),
            }
        };

        Error { code, message }
    }

    pub fn init() -> Result<()> {
        match check_call(unsafe { self::internal::pl_pvcam_init() }) {
            PVResult::Ok => Ok(()),
            PVResult::Err => Err(pvcam_error()),
        }
    }

    pub fn uninit() -> Result<()> {
        match check_call(unsafe { self::internal::pl_pvcam_uninit() }) {
            PVResult::Ok => Ok(()),
            PVResult::Err => Err(pvcam_error()),
        }
    }

    pub fn cam_get_total() -> Result<i16> {
        let mut total_cams: i16 = 0;

        match check_call(unsafe { self::internal::pl_cam_get_total(&mut total_cams) }) {
            PVResult::Ok => Ok(total_cams),
            PVResult::Err => Err(pvcam_error()),
        }
    }

    pub fn cam_get_name(cam_num: i16) -> Result<String> {
        unsafe {
            let buf =
                ffi::CString::from_vec_unchecked(vec![0; self::internal::CAM_NAME_LEN as usize])
                    .into_raw();

            match check_call(self::internal::pl_cam_get_name(cam_num, buf)) {
                PVResult::Ok => Ok(ffi::CString::from_raw(buf).into_string()?),
                PVResult::Err => Err(pvcam_error()),
            }
        }
    }

    pub fn cam_open(cam_name: &str) -> Result<i16> {
        let cam_name = ffi::CString::new(cam_name).expect("Unable to create ptr");
        let mut handle: i16 = -1;

        match check_call(unsafe {
            self::internal::pl_cam_open(
                cam_name.as_ptr(),
                &mut handle,
                self::internal::PL_OPEN_MODES_OPEN_EXCLUSIVE as i16,
            )
        }) {
            PVResult::Ok => Ok(handle),
            PVResult::Err => Err(pvcam_error()),
        }
    }

    #[repr(u32)]
    #[derive(Debug, Clone)]
    pub enum Parameter {
        CameraSerial = self::internal::PARAM_HEAD_SER_NUM_ALPHA,
        GainIndex = self::internal::PARAM_GAIN_INDEX,
        ReadoutPort = self::internal::PARAM_READOUT_PORT,
        SensorParallelSize = self::internal::PARAM_PAR_SIZE,
        SensorSerialSize = self::internal::PARAM_SER_SIZE,
        SpeedTableIndex = self::internal::PARAM_SPDTAB_INDEX,
    }

    #[repr(i16)]
    pub enum ParamAttrKind {
        Current = self::internal::PL_PARAM_ATTRIBUTES_ATTR_CURRENT as i16,
        // Count = self::internal::PL_PARAM_ATTRIBUTES_ATTR_COUNT as i16,
        AttrType = self::internal::PL_PARAM_ATTRIBUTES_ATTR_TYPE as i16,
        // Min = self::internal::PL_PARAM_ATTRIBUTES_ATTR_MIN as i16,
        // Max = self::internal::PL_PARAM_ATTRIBUTES_ATTR_MAX as i16,
        // Def = self::internal::PL_PARAM_ATTRIBUTES_ATTR_DEFAULT as i16,
        // Increment = self::internal::PL_PARAM_ATTRIBUTES_ATTR_INCREMENT as i16,
        // Access = self::internal::PL_PARAM_ATTRIBUTES_ATTR_ACCESS as i16,
        Available = self::internal::PL_PARAM_ATTRIBUTES_ATTR_AVAIL as i16,
    }

    fn is_param_avail(cam_handle: i16, param_id: u32) -> Result<bool> {
        unsafe {
            // assume false
            let mut avail: c_types::c_ushort = 0;
            let mut_ptr = &mut avail as *mut c_types::c_ushort as *mut c_types::c_void;

            match check_call(self::internal::pl_get_param(
                cam_handle,
                param_id,
                ParamAttrKind::Available as i16,
                mut_ptr,
            )) {
                PVResult::Ok => {
                    Ok(*(mut_ptr as *const c_types::c_ushort) as u32 == self::internal::PV_OK)
                }
                PVResult::Err => Err(pvcam_error()),
            }
        }
    }

    enum ParamType {
        Int32,
        String,
    }

    fn get_param_type(cam_handle: i16, param_id: u32) -> Result<ParamType> {
        let kind: u32 = unsafe {
            let mut t: c_types::c_ushort = 0;
            let mut_ptr = &mut t as *mut c_types::c_ushort as *mut c_types::c_void;
            match check_call(self::internal::pl_get_param(
                cam_handle,
                param_id,
                ParamAttrKind::AttrType as i16,
                mut_ptr,
            )) {
                PVResult::Ok => *(mut_ptr as *const c_types::c_uint),
                PVResult::Err => {
                    return Err(pvcam_error());
                }
            }
        };

        match kind {
            self::internal::TYPE_UNS16 => Ok(ParamType::Int32), // is this a leaky abstraction?
            self::internal::TYPE_CHAR_PTR => Ok(ParamType::String),
            _ => Err(Error {
                code: -1,
                message: format!("{:#X} unknown parameter type", kind),
            }),
        }
    }

    fn get_param_as_string(
        cam_handle: i16,
        param_id: u32,
        param_attr: ParamAttrKind,
    ) -> Result<String> {
        unsafe {
            let buf =
                ffi::CString::from_vec_unchecked(vec![0; self::internal::MAX_PP_NAME_LEN as usize])
                    .into_raw();

            match check_call(self::internal::pl_get_param(
                cam_handle,
                param_id,
                param_attr as i16,
                buf as *mut c_types::c_void,
            )) {
                PVResult::Ok => Ok(ffi::CString::from_raw(buf).into_string()?),
                PVResult::Err => Err(pvcam_error()),
            }
        }
    }

    fn get_param_as_int32(
        cam_handle: i16,
        param_id: u32,
        param_attr: ParamAttrKind,
    ) -> Result<i32> {
        unsafe {
            let mut value: i32 = 0;
            let mut_ptr = &mut value as *mut c_types::c_int as *mut c_types::c_void;
            match check_call(self::internal::pl_get_param(
                cam_handle,
                param_id,
                param_attr as i16,
                mut_ptr,
            )) {
                PVResult::Ok => Ok(value),
                PVResult::Err => Err(pvcam_error()),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum ParameterValue {
        IntValue(i32),
        StringValue(String),
    }

    pub fn get_param(
        cam_handle: i16,
        parameter: Parameter,
        param_attr: ParamAttrKind,
    ) -> Result<ParameterValue> {
        let param_id = parameter as u32;
        // is_param_avail can succeed with a false value
        if !is_param_avail(cam_handle, param_id)? {
            return Err(Error {
                code: -1,
                message: format!("parameter {} is unknown", param_id),
            });
        }

        match get_param_type(cam_handle, param_id)? {
            ParamType::Int32 => Ok(ParameterValue::IntValue(get_param_as_int32(
                cam_handle, param_id, param_attr,
            )?)),
            ParamType::String => Ok(ParameterValue::StringValue(get_param_as_string(
                cam_handle, param_id, param_attr,
            )?)),
        }
    }
}

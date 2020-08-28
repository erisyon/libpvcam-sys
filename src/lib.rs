pub mod pvcam {
    use std::ffi;
    use std::fmt;
    use std::os::raw as c_types;

    mod internal {
        #![allow(non_upper_case_globals)]
        #![allow(non_camel_case_types)]
        #![allow(non_snake_case)]
        #![allow(dead_code)]
        include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

        extern "C" {
            pub fn pl_cam_open(
                camera_name: *const std::os::raw::c_char,
                hcam: *mut i16,
                o_mode: i16,
            ) -> rs_bool;
        }
    }

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
}

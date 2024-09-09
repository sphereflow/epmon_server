#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Command {
    GetIntervalms = 0x0,
    GetVoltageBufferSize,
    GetBattery1Buffer,
    GetBatteryPackBuffer,
    GetPVBuffer,
    RetransmitBuffers,
    ModbusGetHoldings {
        register_address: u16,
        size: u8,
    },
    ModbusGetInputRegisters {
        register_address: u16,
        size: u8,
    },
    /// sets all the holding values at once
    ModbusSetHoldings {
        register_address: u16,
        new_holding_values: [u16; 15],
    },
}

impl Command {
    pub fn to_bytes(&self) -> [u8; 33] {
        let mut res = [0; 33];
        res[0] = self.discriminant();
        match self {
            Command::ModbusGetInputRegisters {
                register_address,
                size,
            }
            | Command::ModbusGetHoldings {
                register_address,
                size,
            } => {
                let [b1, b2] = register_address.to_be_bytes();
                let b3 = *size;
                res[1] = b1;
                res[2] = b2;
                res[3] = b3;
            }
            Command::ModbusSetHoldings {
                register_address,
                new_holding_values,
            } => {
                let [b1, b2] = register_address.to_be_bytes();
                res[1] = b1;
                res[2] = b2;
                let holding_bytes = &mut res[3..];
                for (ix, val) in new_holding_values.iter().enumerate() {
                    let chunk = val.to_be_bytes();
                    holding_bytes[ix * 2] = chunk[0];
                    holding_bytes[ix * 2 + 1] = chunk[1];
                }
            }
            _ => {}
        }
        res
    }

    pub fn size(&self) -> u8 {
        match self {
            Command::ModbusGetHoldings { size, .. } => *size,
            Command::ModbusGetInputRegisters { size, .. } => *size,
            _ => 0,
        }
    }

    fn discriminant(&self) -> u8 {
        // WHYYYYYY !!!!!!!!!!!!!!!!!!!!!!
        unsafe { *(self as *const Self as *const u8) }
    }
}

impl TryFrom<&[u8]> for Command {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            [0, ..] => Ok(Command::GetIntervalms),
            [1, ..] => Ok(Command::GetVoltageBufferSize),
            [2, ..] => Ok(Command::GetBattery1Buffer),
            [3, ..] => Ok(Command::GetBatteryPackBuffer),
            [4, ..] => Ok(Command::GetPVBuffer),
            [5, ..] => Ok(Command::RetransmitBuffers),
            [6, h1, h2, h3, ..] => Ok(Command::ModbusGetHoldings {
                register_address: u16::from_be_bytes([*h1, *h2]),
                size: *h3,
            }),
            [7, h1, h2, h3, ..] => Ok(Command::ModbusGetInputRegisters {
                register_address: u16::from_be_bytes([*h1, *h2]),
                size: *h3,
            }),
            [8, h1, h2, new_values @ ..] => {
                let mut new_holding_values = [0; 15];
                for (ix, chunk) in new_values.chunks(2).enumerate() {
                    new_holding_values[ix] = u16::from_be_bytes([chunk[0], chunk[1]]);
                }
                Ok(Command::ModbusSetHoldings {
                    register_address: u16::from_be_bytes([*h1, *h2]),
                    new_holding_values,
                })
            }
            _ => Err(()),
        }
    }
}

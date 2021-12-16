use clap::{App, Arg, ArgMatches, SubCommand};
use std::str::FromStr;

use thiserror::Error;

use super::{clap_arg_puzzle_part_time_two, read_file_contents, ReadFileContentsError};

pub const SUBCOMMAND_NAME: &str = "day16";

pub fn subcommand() -> App<'static, 'static> {
    SubCommand::with_name(SUBCOMMAND_NAME)
        .about("My solution for Day 16: Packet Decoder")
        .arg(
            Arg::with_name("input_file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("sets the input file")
                .default_value("puzzle-inputs/day16-input"),
        )
        .arg(clap_arg_puzzle_part_time_two())
}

pub fn handle(matches: &ArgMatches) -> Result<(), Day16Error> {
    let input_file = matches.value_of("input_file");
    let file_contents = read_file_contents(input_file)
        .map_err(|error| Day16Error::ReadFileContents(input_file.map(str::to_string), error))?;
    match matches.value_of("puzzle_part").unwrap_or("two") {
        "two" | "2" => {
            let value_of_packet = calculate_value_of_packet(&file_contents)?;
            println!("The value of the packet is {}.", value_of_packet);
        }
        _ => {
            let sum_of_packet_version_numbers =
                calculate_sum_of_packet_version_numbers(&file_contents)?;
            println!(
                "The sum of the packet version numbers is {}.",
                sum_of_packet_version_numbers
            );
        }
    };
    Ok(())
}

#[derive(Debug, Error)]
pub enum Day16Error {
    #[error("Could not read file contents of \"{0:?}\" ({1})")]
    ReadFileContents(Option<String>, #[source] ReadFileContentsError),
    #[error("Could not calculate sum of packet version numbers")]
    CalculateSumOfPacketVersionNumbers(#[from] CalculateSumOfPacketVersionNumbersError),
    #[error("Could not calculate value of packet")]
    CalculateValueOfPacket(#[from] CalculateValueOfPacketError),
}

pub fn calculate_sum_of_packet_version_numbers(
    bits_transmission: &str,
) -> Result<u128, CalculateSumOfPacketVersionNumbersError> {
    Ok(Packet::from_str(bits_transmission)?.sum_versions())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CalculateSumOfPacketVersionNumbersError {
    #[error("Could not parse packet from str ({0})")]
    PacketFromStr(#[from] PacketFromStrError),
}

pub fn calculate_value_of_packet(
    bits_transmission: &str,
) -> Result<u128, CalculateValueOfPacketError> {
    Ok(Packet::from_str(bits_transmission)?.value())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CalculateValueOfPacketError {
    #[error("Could not parse packet from str ({0})")]
    PacketFromStr(#[from] PacketFromStrError),
}

#[derive(Debug, Eq, PartialEq)]
struct Packet {
    version: u8,
    type_: PacketType,
}

impl Packet {
    fn sum_versions(&self) -> u128 {
        self.version as u128
            + match &self.type_ {
                PacketType::LiteralValue { .. } => 0,
                PacketType::Operator { packets, .. } => {
                    packets.iter().map(Packet::sum_versions).sum()
                }
            }
    }

    fn value(&self) -> u128 {
        self.type_.value()
    }
}

#[derive(Debug, Eq, PartialEq)]
enum PacketType {
    LiteralValue {
        value: u128,
    },
    Operator {
        type_: OperatorType,
        length: LengthType,
        packets: Vec<Packet>,
    },
}

impl PacketType {
    fn value(&self) -> u128 {
        match self {
            PacketType::LiteralValue { value } => *value,
            PacketType::Operator { type_, packets, .. } => match type_ {
                OperatorType::Sum => packets.iter().map(|packet| packet.value()).sum(),
                OperatorType::Product => packets.iter().map(|packet| packet.value()).product(),
                OperatorType::Minimum => packets.iter().map(|packet| packet.value()).min().unwrap(),
                OperatorType::Maximum => packets.iter().map(|packet| packet.value()).max().unwrap(),
                OperatorType::GreaterThan => {
                    if packets[0].value() > packets[1].value() {
                        1
                    } else {
                        0
                    }
                }
                OperatorType::LessThan => {
                    if packets[0].value() < packets[1].value() {
                        1
                    } else {
                        0
                    }
                }
                OperatorType::EqualTo => {
                    if packets[0].value() == packets[1].value() {
                        1
                    } else {
                        0
                    }
                }
            },
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
enum OperatorType {
    Sum,
    Product,
    Minimum,
    Maximum,
    GreaterThan,
    LessThan,
    EqualTo,
}

#[derive(Debug, Eq, PartialEq)]
enum LengthType {
    TotalLengthOfAllSubPacketInBits(u128),
    NumberOfSubPackets(u128),
}

impl FromStr for Packet {
    type Err = PacketFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse<F: FnMut(usize) -> Result<Vec<Bit>, PacketFromStrError>>(
            poll_bits: &mut F,
        ) -> Result<(Packet, u128), PacketFromStrError> {
            let mut read_bits = 0;

            let pver = poll_bits(3)?.to_u128()? as u8;
            let tid = poll_bits(3)?.to_u128()? as u8;
            read_bits += 6;

            match tid {
                4 => {
                    let mut value = Vec::new();
                    let mut is_last_block = false;
                    while !is_last_block {
                        is_last_block = matches!(poll_bits(1)?[0], Bit::Zero);
                        value.extend(poll_bits(4)?);
                        read_bits += 5;
                    }
                    Ok((
                        Packet {
                            version: pver,
                            type_: PacketType::LiteralValue {
                                value: value.to_u128()?,
                            },
                        },
                        read_bits,
                    ))
                }
                _ => {
                    let type_ = match tid {
                        0 => OperatorType::Sum,
                        1 => OperatorType::Product,
                        2 => OperatorType::Minimum,
                        3 => OperatorType::Maximum,
                        5 => OperatorType::GreaterThan,
                        6 => OperatorType::LessThan,
                        7 => OperatorType::EqualTo,
                        _ => panic!("did not expect type id of {} here, because it should has been handled before", tid),
                    };
                    let length = match poll_bits(1)?[0] {
                        Bit::Zero => {
                            let length = poll_bits(15)?.to_u128()? as u128;
                            read_bits += 16;
                            LengthType::TotalLengthOfAllSubPacketInBits(length)
                        }
                        Bit::One => {
                            let packet_count = poll_bits(11)?.to_u128()? as u128;
                            read_bits += 12;
                            LengthType::NumberOfSubPackets(packet_count)
                        }
                    };
                    let packets = match length {
                        LengthType::TotalLengthOfAllSubPacketInBits(length) => {
                            let mut read_so_far = 0;
                            let mut packets = Vec::new();
                            while read_so_far < length {
                                let (packet, packet_read_bits) = parse(poll_bits)?;
                                read_bits += packet_read_bits;
                                read_so_far += packet_read_bits;
                                packets.push(packet);
                            }
                            packets
                        }
                        LengthType::NumberOfSubPackets(count) => {
                            let mut packets = Vec::new();
                            for _ in 0..count {
                                let (packet, packet_read_bits) = parse(poll_bits)?;
                                read_bits += packet_read_bits;
                                packets.push(packet);
                            }
                            packets
                        }
                    };
                    Ok((
                        Packet {
                            version: pver,
                            type_: PacketType::Operator {
                                type_,
                                length,
                                packets,
                            },
                        },
                        read_bits,
                    ))
                }
            }
        }

        let mut characters = s.chars().collect::<Vec<char>>();
        let mut bit_buffer = Vec::new();

        let mut poll_bits = |count: usize| -> Result<Vec<Bit>, PacketFromStrError> {
            while bit_buffer.len() < count {
                if characters.is_empty() {
                    return Err(PacketFromStrError::MissingBitsInInput(
                        count - bit_buffer.len(),
                    ));
                } else {
                    bit_buffer.extend(characters.remove(0).to_bits()?);
                }
            }
            Ok(bit_buffer.split_off_head(count))
        };

        parse(&mut poll_bits).map(|(packet, _)| packet)
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum PacketFromStrError {
    #[error("Could not convert char to bits ({0})")]
    CharToBits(#[from] CharToBitsError),
    #[error("Could not convert Bit Vector to u128 ({0})")]
    VecBitToU128(#[from] VecBitToU128Error),
    #[error("Missing {0} bits in input")]
    MissingBitsInInput(usize),
}

trait ToBits {
    type Error;

    fn to_bits(&self) -> Result<Vec<Bit>, Self::Error>;
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Bit {
    Zero,
    One,
}

trait SplitOffHead {
    fn split_off_head(&mut self, at: usize) -> Self;
}

impl SplitOffHead for Vec<Bit> {
    fn split_off_head(&mut self, at: usize) -> Self {
        let tail = self.split_off(at);
        let head = self.clone();
        *self = tail;
        head
    }
}

trait ToU128 {
    type Error;

    fn to_u128(&self) -> Result<u128, Self::Error>;
}

impl ToU128 for Vec<Bit> {
    type Error = VecBitToU128Error;

    fn to_u128(&self) -> Result<u128, Self::Error> {
        if self.len() > 128 {
            Err(VecBitToU128Error::TooManyBits(128))
        } else {
            let mut output = 0;
            for bit in self {
                output <<= 1;
                if matches!(bit, Bit::One) {
                    output |= 1;
                }
            }
            Ok(output)
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum VecBitToU128Error {
    #[error("Encountered too many bits (encountered {0})")]
    TooManyBits(usize),
}

impl ToBits for char {
    type Error = CharToBitsError;

    fn to_bits(&self) -> Result<Vec<Bit>, Self::Error> {
        match self {
            '0' => Ok(vec![Bit::Zero, Bit::Zero, Bit::Zero, Bit::Zero]),
            '1' => Ok(vec![Bit::Zero, Bit::Zero, Bit::Zero, Bit::One]),
            '2' => Ok(vec![Bit::Zero, Bit::Zero, Bit::One, Bit::Zero]),
            '3' => Ok(vec![Bit::Zero, Bit::Zero, Bit::One, Bit::One]),
            '4' => Ok(vec![Bit::Zero, Bit::One, Bit::Zero, Bit::Zero]),
            '5' => Ok(vec![Bit::Zero, Bit::One, Bit::Zero, Bit::One]),
            '6' => Ok(vec![Bit::Zero, Bit::One, Bit::One, Bit::Zero]),
            '7' => Ok(vec![Bit::Zero, Bit::One, Bit::One, Bit::One]),
            '8' => Ok(vec![Bit::One, Bit::Zero, Bit::Zero, Bit::Zero]),
            '9' => Ok(vec![Bit::One, Bit::Zero, Bit::Zero, Bit::One]),
            'A' => Ok(vec![Bit::One, Bit::Zero, Bit::One, Bit::Zero]),
            'B' => Ok(vec![Bit::One, Bit::Zero, Bit::One, Bit::One]),
            'C' => Ok(vec![Bit::One, Bit::One, Bit::Zero, Bit::Zero]),
            'D' => Ok(vec![Bit::One, Bit::One, Bit::Zero, Bit::One]),
            'E' => Ok(vec![Bit::One, Bit::One, Bit::One, Bit::Zero]),
            'F' => Ok(vec![Bit::One, Bit::One, Bit::One, Bit::One]),
            c => Err(CharToBitsError::Unknown(*c)),
        }
    }
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum CharToBitsError {
    #[error("Unknown character '{0}'")]
    Unknown(char),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_from_str_should_return_literal_2021() {
        // given
        let input = "D2FE28";

        // when
        let packet = Packet::from_str(input);

        // then
        assert_eq!(
            packet,
            Ok(Packet {
                version: 6,
                type_: PacketType::LiteralValue { value: 2021 }
            })
        )
    }

    #[test]
    fn packet_from_str_should_return_operator_id_6() {
        // given
        let input = "38006F45291200";

        // when
        let packet = Packet::from_str(input);

        // then
        assert_eq!(
            packet,
            Ok(Packet {
                version: 1,
                type_: PacketType::Operator {
                    type_: OperatorType::LessThan,
                    length: LengthType::TotalLengthOfAllSubPacketInBits(27),
                    packets: vec![
                        Packet {
                            version: 6,
                            type_: PacketType::LiteralValue { value: 10 }
                        },
                        Packet {
                            version: 2,
                            type_: PacketType::LiteralValue { value: 20 }
                        },
                    ],
                },
            })
        )
    }

    #[test]
    fn packet_from_str_should_return_operator_version_7() {
        // given
        let input = "EE00D40C823060";

        // when
        let packet = Packet::from_str(input);

        // then
        assert_eq!(
            packet,
            Ok(Packet {
                version: 7,
                type_: PacketType::Operator {
                    type_: OperatorType::Maximum,
                    length: LengthType::NumberOfSubPackets(3),
                    packets: vec![
                        Packet {
                            version: 2,
                            type_: PacketType::LiteralValue { value: 1 }
                        },
                        Packet {
                            version: 4,
                            type_: PacketType::LiteralValue { value: 2 }
                        },
                        Packet {
                            version: 1,
                            type_: PacketType::LiteralValue { value: 3 }
                        },
                    ],
                },
            })
        )
    }

    #[test]
    fn test_calculate_sum_of_packet_version_numbers() {
        // given
        let input_a = "8A004A801A8002F478";
        let input_b = "620080001611562C8802118E34";
        let input_c = "C0015000016115A2E0802F182340";
        let input_d = "A0016C880162017C3686B18A3D4780";

        // when
        let version_sum_a = calculate_sum_of_packet_version_numbers(input_a);
        let version_sum_b = calculate_sum_of_packet_version_numbers(input_b);
        let version_sum_c = calculate_sum_of_packet_version_numbers(input_c);
        let version_sum_d = calculate_sum_of_packet_version_numbers(input_d);

        // then
        assert_eq!(version_sum_a, Ok(16));
        assert_eq!(version_sum_b, Ok(12));
        assert_eq!(version_sum_c, Ok(23));
        assert_eq!(version_sum_d, Ok(31));
    }

    #[test]
    fn test_calculate_value_of_packet() {
        // given
        let input_a = "C200B40A82";
        let input_b = "04005AC33890";
        let input_c = "880086C3E88112";
        let input_d = "CE00C43D881120";
        let input_e = "D8005AC2A8F0";
        let input_f = "F600BC2D8F";
        let input_g = "9C005AC2F8F0";
        let input_h = "9C0141080250320F1802104A08";

        // when
        let value_a = calculate_value_of_packet(input_a);
        let value_b = calculate_value_of_packet(input_b);
        let value_c = calculate_value_of_packet(input_c);
        let value_d = calculate_value_of_packet(input_d);
        let value_e = calculate_value_of_packet(input_e);
        let value_f = calculate_value_of_packet(input_f);
        let value_g = calculate_value_of_packet(input_g);
        let value_h = calculate_value_of_packet(input_h);

        // then
        assert_eq!(value_a, Ok(3));
        assert_eq!(value_b, Ok(54));
        assert_eq!(value_c, Ok(7));
        assert_eq!(value_d, Ok(9));
        assert_eq!(value_e, Ok(1));
        assert_eq!(value_f, Ok(0));
        assert_eq!(value_g, Ok(0));
        assert_eq!(value_h, Ok(1));
    }
}

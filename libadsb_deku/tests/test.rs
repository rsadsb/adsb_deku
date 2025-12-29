use adsb_deku::adsb::{VerticalRateSource, ME};
use adsb_deku::{CPRFormat, Capability, Frame, DF};
use assert_hex::assert_eq_hex;
use hexlit::hex;
use test_log::test;

#[test]
fn testing01() {
    // from adsb-rs
    let bytes = hex!("8D40621D58C382D690C8AC2863A7");
    let frame = Frame::from_bytes(&bytes);
    if let DF::ADSB(adsb) = frame.unwrap().df {
        if let ME::AirbornePositionBaroAltitude { id: _, altitude } = adsb.me {
            assert_eq!(altitude.alt, Some(38000));
            assert_eq!(altitude.lat_cpr, 93000);
            assert_eq!(altitude.lon_cpr, 51372);
            assert_eq!(altitude.odd_flag, CPRFormat::Even);
            return;
        }
    }
    unreachable!();
}

#[test]
fn testing02() {
    // from adsb-rs
    let bytes = hex!("8da3d42599250129780484712c50");
    let frame = Frame::from_bytes(&bytes);
    if let DF::ADSB(adsb) = frame.unwrap().df {
        if let ME::AirborneVelocity(me) = adsb.me {
            let (heading, ground_speed, vertical_rate) = me.calculate().unwrap();
            assert!((heading - 322.197_2).abs() < f32::EPSILON);
            assert!((ground_speed - 417.655_360_315_176_6).abs() < f64::EPSILON);
            assert_eq!(vertical_rate, 0);
            assert_eq!(me.vrate_src, VerticalRateSource::GeometricAltitude);
            return;
        }
    }
    unreachable!();
}

#[test]
fn testing03() {
    // from dump1090
    // *8da08f94ea1b785e8f3c088ab467;
    // CRC: 000000
    // RSSI: -30.2 dBFS
    // Score: 1800
    // Time: 100330060143.92us
    // DF:17 AA:A08F94 CA:5 ME:EA1B785E8F3C08
    //  Extended Squitter Target state and status (V2) (29/1) (reliable)
    //   ICAO Address:  A08F94 (Mode S / ADS-B)
    //   Air/Ground:    airborne
    //   NIC-baro:      1
    //   NACp:          9
    //   SIL:           3 (p <= 0.00001%, unknown type)
    //   Selected heading:        229.9
    //   MCP selected altitude:   14016 ft
    //   QNH:                     1012.8 millibars
    let bytes = hex!("8da08f94ea1b785e8f3c088ab467");
    let frame = Frame::from_bytes(&bytes);
    if let DF::ADSB(adsb) = frame.unwrap().df {
        if let ME::TargetStateAndStatusInformation(me) = adsb.me {
            assert_eq!(me.subtype, 1);
            assert!(!me.is_fms);
            assert_eq!(me.altitude, 14016);
            assert!((me.qnh - 1012.8).abs() < f32::EPSILON);
            assert!(me.is_heading);
            assert!((me.heading - 229.92188).abs() < f32::EPSILON);
            assert_eq!(me.nacp, 9);
            assert_eq!(me.nicbaro, 1);
            assert_eq!(me.sil, 3);
            assert!(!me.mode_validity);
            return;
        }
    }
    unreachable!();
}

// dump1090
//
// *8dacc040f8210002004ab8569c35;
// CRC: 000000
// RSSI: -32.5 dBFS
// Score: 1800
// Time: 709947330.42us
// DF:17 AA:ACC040 CA:5 ME:F8210002004AB8
//  Extended Squitter Aircraft operational status (airborne) (31/0) (reliable)
//   ICAO Address:  ACC040 (Mode S / ADS-B)
//   Air/Ground:    airborne
//   NIC-A:         0
//   NIC-baro:      1
//   NACp:          10
//   GVA:           2
//   SIL:           3 (p <= 0.00001%, per flight hour)
//   SDA:           2
//   Aircraft Operational Status:
//     Version:            2
//     Capability classes: ACAS TS
//     Operational modes:
//     Heading ref dir:    True heading
#[test]
fn testing04() {
    // TODO
    let bytes = hex!("8dacc040f8210002004ab8569c35");
    let frame = Frame::from_bytes(&bytes).unwrap();
    if let DF::ADSB(adsb) = frame.df {
        assert_eq_hex!(adsb.icao.0, [0xac, 0xc0, 0x40]);
        assert_eq!(adsb.capability, Capability::AG_AIRBORNE);
        return;
    }
    unreachable!();
}

// *5dab3d17d4ba29;
// CRC: 000001
// RSSI: -3.5 dBFS
// Score: 1000
// Time: 1352791.42us
// DF:11 AA:AB3D17 IID:1 CA:5
//  All Call Reply
//    ICAO Address:  AB3D17 (Mode S / ADS-B)
//      Air/Ground:    airborne
#[test]
fn testing05() {
    let bytes = hex!("5dab3d17d4ba29");
    let frame = Frame::from_bytes(&bytes).unwrap();
    if let DF::AllCallReply { icao, capability, .. } = frame.df {
        assert_eq_hex!(icao.0, hex!("ab3d17"));
        assert_eq!(capability, Capability::AG_AIRBORNE);
        return;
    }
    unreachable!();
}

// *8dab3d17ea486860015f4870b796;
// CRC: 000000
// RSSI: -3.5 dBFS
// Score: 1800
// Time: 985167.50us
// DF:17 AA:AB3D17 CA:5 ME:EA486860015F48
//  Extended Squitter Target state and status (V2) (29/1) (reliable)
//   ICAO Address:  AB3D17 (Mode S / ADS-B)
//   Air/Ground:    airborne
//   NIC-baro:      1
//   NACp:          10
//   SIL:           3 (p <= 0.00001%, unknown type)
//   MCP selected altitude:   37024 ft
//   QNH:                     1013.6 millibars
//   Nav modes:               autopilot althold tcas
#[test]
fn testing06() {
    let bytes = hex!("8dab3d17ea486860015f4870b796");
    let frame = Frame::from_bytes(&bytes).unwrap();
    if let DF::ADSB(adsb) = frame.df {
        if let ME::TargetStateAndStatusInformation(me) = adsb.me {
            assert_eq!(me.subtype, 1);
            assert!(!me.is_fms);
            assert_eq!(me.altitude, 37024);
            assert!((me.qnh - 1013.6).abs() < f32::EPSILON);
            assert!(!me.is_heading);
            assert!((me.heading - 0.0).abs() < f32::EPSILON);
            assert_eq!(me.nacp, 10);
            assert_eq!(me.nicbaro, 1);
            assert_eq!(me.sil, 3);
            assert!(me.mode_validity);
            return;
        }
    }
    unreachable!();
}

// *5da039b46d7d81;
// CRC: 000000
// RSSI: -13.9 dBFS
// Score: 750
// Time: 183194.00us
// DF:11 AA:A039B4 IID:0 CA:5
//  All Call Reply (reliable)
//   ICAO Address:  A039B4 (Mode S / ADS-B)
//   Air/Ground:    airborne
#[test]
fn testing08() {
    let bytes = hex!("5da039b46d7d81");
    let frame = Frame::from_bytes(&bytes).unwrap();
    if let DF::AllCallReply { icao, capability, .. } = frame.df {
        assert_eq_hex!(icao.0, hex!("a039b4"));
        assert_eq!(capability, Capability::AG_AIRBORNE);
        return;
    }
    unreachable!();
}

//*02e19cb02512c3;
//CRC: 0d097e
//RSSI: -8.1 dBFS
//Score: 1000
//Time: 91219304.17us
//DF:0 addr:0D097E VS:0 CC:1 SL:7 RI:3 AC:7344
// Short Air-Air Surveillance
//  ICAO Address:  0D097E (Mode S / ADS-B)
//  Air/Ground:    airborne?
//  Altitude:      45000 ft barometric
#[test]
fn testing_df_shortairairsurveillance() {
    let bytes = hex!("02e19cb02512c3");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Short Air-Air Surveillance
  ICAO Address:  0d097e (Mode S / ADS-B)
  Air/Ground:    airborne?
  Altitude:      45000 ft barometric
"#,
        resulting_string
    );
}

// -----new-----
// ---deku
// Frame {
//    df: ADSB {
//        capability: AG_AIRBORNE,
//        icao: [
//            13,
//            9,
//            126,
//        ],
//        me: AircraftOperationStatus(
//            Airborne(
//                OperationStatusAirborne {
//                    capacity_class_codes: 35,
//                    operational_mode_codes: 7,
//                    version_number: DOC9871AppendixC,
//                    nic_supplement_a: 1,
//                    navigational_accuracy_category: 10,
//                    geometric_vertical_accuracy: 1,
//                    source_integrity_level: 1,
//                    barometric_altitude_integrity: 1,
//                    horizontal_reference_direction: 1,
//                    sil_supplement: 0,
//                    reserved: 0,
//                },
//            ),
//        ),
//        pi: 3422506,
//    },
//    crc: 0,
//}
// ---regular
// *8d0d097ef8230007005ab8547268;
// CRC: 000000
// RSSI: -10.3 dBFS
// Score: 1800
// Time: 92723308.25us
// DF:17 AA:0D097E CA:5 ME:F8230007005AB8
//  Extended Squitter Aircraft operational status (airborne) (31/0)
//   ICAO Address:  0D097E (Mode S / ADS-B)
//   Air/Ground:    airborne
//   Aircraft Operational Status:
//     Version:            2
//     Capability classes: ACAS ARV TS
//     Operational modes:  SAF SDA=3
//     NIC-A:              1
//     NACp:               10
//     GVA:                2
//     SIL:                3 (per hour)
//     NICbaro:            1
//     Heading reference:  true north
#[test]
fn testing_df_extendedsquitteraircraftopstatus() {
    let bytes = hex!("8d0d097ef8230007005ab8547268");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Aircraft operational status (airborne)
  Address:       0d097e (Mode S / ADS-B)
  Air/Ground:    airborne
  Aircraft Operational Status:
   Version:            2
   Capability classes: ACAS ARV TS
   Operational modes:  SAF SDA=3
   NIC-A:              1
   NACp:               10
   GVA:                2
   SIL:                3 (per hour)
   NICbaro:            1
   Heading reference:  true north
"#,
        resulting_string
    );

    let bytes = hex!("8da1a8daf82300060049b870c88b");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Aircraft operational status (airborne)
  Address:       a1a8da (Mode S / ADS-B)
  Air/Ground:    airborne
  Aircraft Operational Status:
   Version:            2
   Capability classes: ACAS ARV TS
   Operational modes:  SAF SDA=2
   NIC-A:              0
   NACp:               9
   GVA:                2
   SIL:                3 (per hour)
   NICbaro:            1
   Heading reference:  true north
"#,
        resulting_string
    );
}

#[test]
fn testing_allcall_reply() {
    let bytes = hex!("5da58fd4561b39");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" All Call Reply
  ICAO Address:  a58fd4 (Mode S / ADS-B)
  Air/Ground:    airborne
"#,
        resulting_string
    );
}

#[test]
fn testing_airbornepositionbaroaltitude() {
    let bytes = hex!("8da2c1bd587ba2adb31799cb802b");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Airborne position (barometric altitude)
  Address:       a2c1bd (Mode S / ADS-B)
  Air/Ground:    airborne
  Altitude:      23650 ft barometric
  CPR type:      Airborne
  CPR odd flag:  even
  CPR latitude:  (87769)
  CPR longitude: (71577)
"#,
        resulting_string
    );
}

#[test]
fn testing_surveillancealtitudereply() {
    let bytes = hex!("200012b0d96e39");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Surveillance, Altitude Reply
  ICAO Address:  a3ecce (Mode S / ADS-B)
  Air/Ground:    airborne?
  Altitude:      29000 ft barometric
"#,
        resulting_string
    );
}

// #[test]
// fn testing_surveillanceidentityreply_err() {
//     let bytes = hex!("245093892a1bfd");
//     let frame = Frame::from_bytes(&bytes).unwrap();
//     let resulting_string = format!("{frame}");
//     assert_eq!(
//         r#" Surveillance, Altitude Reply
//   ICAO Address:  a168ad (Mode S / ADS-B)
//   Air/Ground:    airborne?
//   Altitude:      6500 ft barometric
// "#,
//         resulting_string
//     );
// }

// TODO
// This test is from mode-s.org, check with the dump1090-rs
#[test]
fn testing_surveillanceidentityreply() {
    let bytes = hex!("2A00516D492B80");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Surveillance, Identity Reply
  ICAO Address:  510af9 (Mode S / ADS-B)
  Air/Ground:    airborne
  Identity:      0356
"#,
        resulting_string
    );
}

#[test]
fn testing_airbornevelocity() {
    let bytes = hex!("8dac8e1a9924263950043944cf32");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Airborne velocity over ground, subsonic
  Address:       ac8e1a (Mode S / ADS-B)
  Air/Ground:    airborne
  GNSS delta:    1400 ft
  Heading:       356
  Speed:         458 kt groundspeed
  Vertical rate: 0 ft/min GNSS
"#,
        resulting_string
    );

    let bytes = hex!("8da3f9cb9910100da8148571db11");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Airborne velocity over ground, subsonic
  Address:       a3f9cb (Mode S / ADS-B)
  Air/Ground:    airborne
  GNSS delta:    -100 ft
  Heading:       8
  Speed:         109 kt groundspeed
  Vertical rate: -256 ft/min barometric
"#,
        resulting_string
    );
}

#[test]
fn testing_targetstateandstatusinformation() {
    let bytes = hex!("8da97753ea2d0858015c003ee5de");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Target state and status (V2)
  Address:       a97753 (Mode S / ADS-B)
  Air/Ground:    airborne
  Target State and Status:
    Target altitude:   MCP, 23008 ft
    Altimeter setting: 1012.8 millibars
    ACAS:              NOT operational
    NACp:              10
    NICbaro:           1
    SIL:               3 (per sample)
    QNH:               1012.8 millibars
"#,
        resulting_string
    );
}

#[test]
fn testing_aircraftidentificationandcategory() {
    let bytes = hex!("8da3f9cb213b3d75c1582080f4d9");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Aircraft identification and category
  Address:       a3f9cb (Mode S / ADS-B)
  Air/Ground:    airborne
  Ident:         N3550U
  Category:      A1
"#,
        resulting_string
    );
}

#[test]
fn testing_issue_01() {
    let bytes = hex!("8dad50a9ea466867811c08abbaa2");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Target state and status (V2)
  Address:       ad50a9 (Mode S / ADS-B)
  Air/Ground:    airborne
  Target State and Status:
    Target altitude:   MCP, 36000 ft
    Altimeter setting: 1013.6 millibars
    Target heading:    315
    ACAS:              operational 
    NACp:              8
    NICbaro:           1
    SIL:               3 (per sample)
    QNH:               1013.6 millibars
"#,
        resulting_string
    );
}

#[test]
fn testing_issue_03() {
    let bytes = hex!("80e1969058b5025b9850641d2974");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Long Air-Air ACAS
  ICAO Address:  ac049e (Mode S / ADS-B)
  Air/Ground:    airborne?
  Baro altitude: 35000 ft
"#,
        resulting_string
    );
}

#[test]
fn testing_issue_04() {
    let bytes = hex!("0621776e99b6ad");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Short Air-Air Surveillance
  ICAO Address:  a33325 (Mode S / ADS-B)
  Air/Ground:    ground
"#,
        resulting_string
    );
}

#[test]
fn testing_df_21() {
    let bytes = hex!("AE24238D15EE315463718B1AF755");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Comm-B, Identity Reply
    ICAO Address:  a95fdc (Mode S / ADS-B)
    Squawk:        6246
    Comm-B format: unknown format
"#,
        resulting_string
    );
}

#[test]
fn testing_df_27() {
    let bytes = hex!("daca7f82613c2db14a49c535a3a2");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Mode S Extended Squitter Message
    ICAO Address:     a01f73 (Mode S / ADS-B)
"#,
        resulting_string
    );
}

#[test]
fn testing_df_18() {
    // test github issue #2 (with sample output from dump1090_fa as control)
    let bytes = hex!("95298FCA680946499671468C7ACA");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter (Non-Transponder) Airborne position (barometric altitude)
  Address:       298fca (TIS-B)
  Air/Ground:    airborne?
  Altitude:      700 ft barometric
  CPR type:      Airborne
  CPR odd flag:  odd
  CPR latitude:  (74955)
  CPR longitude: (28998)
"#,
        resulting_string
    );

    // test github issue #3 (with sample output from dump1090_fa as control)
    let bytes = hex!("96A082FB213B1CF2113820D6EDDF");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter (Non-Transponder) Aircraft identification and category
  Address:       a082fb (ADS-R)
  Air/Ground:    airborne?
  Ident:         N132DS
  Category:      A1
"#,
        resulting_string
    );

    // test github issue #4 (with sample output from dump1090_fa as control)
    let bytes = hex!("96A6C24699141E0E8018074AA959");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter (Non-Transponder) Airborne velocity over ground, subsonic
  Address:       a6c246 (ADS-R)
  Air/Ground:    airborne?
  GNSS delta:    150 ft
  Heading:       346
  Speed:         118 kt groundspeed
  Vertical rate: 320 ft/min barometric
"#,
        resulting_string
    );

    // test github issue #5 (with sample output from dump1090_fa as control)
    let bytes = hex!("92A24528993C238900062053CDEF");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter (Non-Transponder) Airborne velocity over ground, subsonic
  Address:       a24528 (TIS-B)
  Air/Ground:    airborne?
  GNSS delta:    775 ft
  Heading:       206
  Speed:         78 kt groundspeed
  Vertical rate: 0 ft/min barometric
"#,
        resulting_string
    );

    // test github issue #6 (with sample output from dump1090_fa as control)
    let bytes = hex!("96130D9D910F86188A7A71EF6DCB");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter (Non-Transponder) Airborne position (barometric altitude)
  Address:       130d9d (ADS-R)
  Air/Ground:    airborne?
  Altitude:      2000 ft barometric
  CPR type:      Airborne
  CPR odd flag:  odd
  CPR latitude:  (68677)
  CPR longitude: (31345)
"#,
        resulting_string
    );

    // test github issue #7 (with sample output from dump1090_fa as control)
    let bytes = hex!("91ADF9CEC11C0524407F11538EE5");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter (Non-Transponder) Reserved for surface system status
  Address:       adf9ce (ADS-B)
  Air/Ground:    airborne?
"#,
        resulting_string
    );

    let bytes = hex!("97CAEEF737FB1341BF58DF19118A");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter (Non-Transponder) Surface position
  Address:       caeef7 (unknown addressing scheme)
"#,
        resulting_string
    );

    // test github issue #8 (with sample output from dump1090_fa as control)
    let bytes = hex!("96A4D01FF900210600493075E234");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter (Non-Transponder) Aircraft operational status (surface)
  Address:       a4d01f (ADS-R)
  Air/Ground:    airborne?
  Aircraft Operational Status:
   Version:            2
   NIC-A:              0
   NIC-C:              0
   NACv:               1
   Capability classes: L/W=1
   Operational modes:  SAF SDA=2
   NACp:               9
   SIL:                3 (per hour)
   NICbaro:            0
   Heading reference:  true north
"#,
        resulting_string
    );
}
#[test]
fn test_emergency() {
    let bytes = hex!("8dc06800e1108500000000baa81f");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Emergency/priority status
  Address:       c06800 (Mode S / ADS-B)
  Air/Ground:    airborne
  Squawk:        4016
  Emergency/priority:    no emergency
"#,
        resulting_string
    );
}

#[test]
fn issue_10() {
    let bytes = hex!("8DA35EBC9B000024B00C0004E897");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Airspeed and heading, subsonic
  Address:       a35ebc (Mode S / ADS-B)
  Air/Ground:    airborne
  IAS:           292 kt
  Baro rate:     128 ft/min
  NACv:          0
"#,
        resulting_string
    );
}

#[test]
fn issue_11_12() {
    let bytes = hex!("8da90a6e000000000000005cab8b");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter No position information
  Address:       a90a6e (Mode S / ADS-B)
  Air/Ground:    airborne
"#,
        resulting_string
    );

    let bytes = hex!("92ef92b301154cb9ab09466702c6");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter (Non-Transponder) No position information
  Address:       ef92b3 (TIS-B)
  Air/Ground:    airborne?
"#,
        resulting_string
    );
}

#[test]
fn fix_issue_unknown() {
    let bytes = hex!("8d85d792beaf5654b710d87357ee");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Unknown
  Address:       85d792 (Mode S / ADS-B)
  Air/Ground:    airborne
"#,
        resulting_string
    );

    let bytes = hex!("972ae8d6d73e298fcaa6bec4c338");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter (Non-Transponder) Unknown
  Address:       2ae8d6 (unknown addressing scheme)
  Air/Ground:    airborne?
"#,
        resulting_string
    );
}

#[test_log::test]
fn fix_issue_13() {
    // 1
    let bytes = hex!("8dab92a2593e0664204c69d8fe84");
    let frame = Frame::from_bytes(&bytes).unwrap();
    println!("{frame:02x?}");
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Airborne position (barometric altitude)
  Address:       ab92a2 (Mode S / ADS-B)
  Air/Ground:    airborne
  Altitude:      10600 ft barometric
  CPR type:      Airborne
  CPR odd flag:  odd
  CPR latitude:  (78352)
  CPR longitude: (19561)
"#,
        resulting_string
    );

    // 2
    let bytes = hex!("8dab92a299105e93001486608c6d");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Airborne velocity over ground, subsonic
  Address:       ab92a2 (Mode S / ADS-B)
  Air/Ground:    airborne
  GNSS delta:    -125 ft
  Heading:       149
  Speed:         177 kt groundspeed
  Vertical rate: 256 ft/min barometric
"#,
        resulting_string
    );

    // 3
    let bytes = hex!("020007a0d08ff4");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Short Air-Air Surveillance
  ICAO Address:  ab92a2 (Mode S / ADS-B)
  Air/Ground:    airborne?
  Altitude:      10600 ft barometric
"#,
        resulting_string
    );

    // 4
    let bytes = hex!("5dab92a2b04912");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" All Call Reply
  ICAO Address:  ab92a2 (Mode S / ADS-B)
  Air/Ground:    airborne
"#,
        resulting_string
    );

    // 4
    let bytes = hex!("8dab92a2593e0664204c69d8fe84");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Airborne position (barometric altitude)
  Address:       ab92a2 (Mode S / ADS-B)
  Air/Ground:    airborne
  Altitude:      10600 ft barometric
  CPR type:      Airborne
  CPR odd flag:  odd
  CPR latitude:  (78352)
  CPR longitude: (19561)
"#,
        resulting_string
    );
}

#[test]
fn test_issue_14() {
    let bytes = hex!("a0001910204d7075d35820c25c0c");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Comm-B, Altitude Reply
  ICAO Address:  aa6f80 (Mode S / ADS-B)
  Altitude:      39000 ft
  Comm-B format: BDS2,0 Aircraft identification
  Ident:         SWA545
"#,
        resulting_string
    );

    let bytes = hex!("a000171810030a80f6000012bd7b");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Comm-B, Altitude Reply
  ICAO Address:  aacb19 (Mode S / ADS-B)
  Altitude:      36000 ft
  Comm-B format: BDS1,0 Datalink capabilities
"#,
        resulting_string
    );
}

#[test]
fn test_issue_09() {
    let bytes = hex!("a00017b010030a80f60000a0fc1e");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Comm-B, Altitude Reply
  ICAO Address:  a6c756 (Mode S / ADS-B)
  Altitude:      37000 ft
  Comm-B format: BDS1,0 Datalink capabilities
"#,
        resulting_string
    );

    let bytes = hex!("a000179f0000000000000019a524");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Comm-B, Altitude Reply
  ICAO Address:  a6c756 (Mode S / ADS-B)
  Altitude:      36975 ft
  Comm-B format: empty response
"#,
        resulting_string
    );
}

#[test]
fn test_issue_16() {
    let bytes = hex!("a227ed3417826515bebd01707629");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Comm-B, Altitude Reply
  ICAO Address:  abef98 (Mode S / ADS-B)
  Altitude:      20300 ft
  Comm-B format: unknown format
"#,
        resulting_string
    );
}

#[test]
fn test_operational_coordination() {
    let bytes = hex!("9143e8eef79baeeacca522b044bf");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter (Non-Transponder) Aircraft Operational Coordination
  Address:       43e8ee (ADS-B)
"#,
        resulting_string
    );
}

#[test]
fn test_issue_25() {
    let bytes = hex!("92479249fcb22e16fbdc3bac5b56");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter (Non-Transponder) Aircraft operational status (reserved)
  Address:       479249 (TIS-B)
"#,
        resulting_string
    );
}

#[test]
fn test_issue_22() {
    let bytes = hex!("911c059d9805a452cf109f64924f");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter (Non-Transponder) Airborne Velocity status (reserved)
  Address:       1c059d (ADS-B)
"#,
        resulting_string
    );
}

#[test]
fn test_df17_error() {
    let bytes = hex!("8da04e60ea3ab860015f889746a9");
    let frame = Frame::from_bytes(&bytes).unwrap();
    let resulting_string = format!("{frame}");
    assert_eq!(
        r#" Extended Squitter Target state and status (V2)
  Address:       a04e60 (Mode S / ADS-B)
  Air/Ground:    airborne
  Target State and Status:
    Target altitude:   MCP, 30016 ft
    Altimeter setting: 1013.6 millibars
    ACAS:              operational autopilot vnav 
    NACp:              10
    NICbaro:           1
    SIL:               3 (per sample)
    QNH:               1013.6 millibars
"#,
        resulting_string
    );
}

mod handwritten {
    extern crate test;
    use super::super::{FileData, Occurrence, Skull, WithId};

    type SkullId = <Skull as super::super::Data>::Id;
    type OccurrenceId = <Occurrence as super::super::Data>::Id;

    #[bench]
    fn serialize_skull(bench: &mut test::Bencher) {
        let skull = Skull {
            name: String::from("xnamex"),
            color: String::from("xcolorx"),
            icon: String::from("xiconx"),
            unit_price: 0.1,
            limit: None,
        };

        bench.iter(|| {
            let mut buffer = vec![];

            (0..100)
                .map(|i| SkullId::new(i, skull.clone()))
                .for_each(|s| Skull::write_tsv(s, &mut buffer).unwrap());
        });
    }

    #[bench]
    fn deserialize_skull(bench: &mut test::Bencher) {
        let data = (0..100)
            .map(|i| format!("{i}\txnamex\txcolorx\txiconx\t1.2\t{i}"))
            .collect::<Vec<_>>();

        bench.iter(|| {
            let data = data.clone();

            for (i, string) in data.into_iter().enumerate() {
                let s = Skull::read_tsv(Ok(string)).unwrap();
                assert_eq!(s.id, i as u32);
                assert_eq!(s.name, "xnamex");
                assert_eq!(s.color, "xcolorx");
                assert_eq!(s.icon, "xiconx");
                assert_eq!(s.unit_price, 1.2);
                assert_eq!(s.limit.unwrap(), i as f32);
            }
        });
    }

    #[bench]
    fn serialize_occurrence(bench: &mut test::Bencher) {
        let occurrence = Occurrence {
            skull: 0,
            amount: 1.2,
            millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        };

        bench.iter(|| {
            let mut buffer = vec![];

            (0..100)
                .map(|i| OccurrenceId::new(i, occurrence.clone()))
                .for_each(|s| Occurrence::write_tsv(s, &mut buffer).unwrap());
        });
    }

    #[bench]
    fn deserialize_occurrence(bench: &mut test::Bencher) {
        let data = (0..100)
            .map(|i| format!("{i}\t0\t1.2\t4"))
            .collect::<Vec<_>>();

        bench.iter(|| {
            let data = data.clone();

            for (i, string) in data.into_iter().enumerate() {
                let s = Occurrence::read_tsv(Ok(string)).unwrap();
                assert_eq!(s.id, i as u32);
                assert_eq!(s.skull, 0);
                assert_eq!(s.amount, 1.2);
                assert_eq!(s.millis, 4);
            }
        });
    }
}

mod serde {
    extern crate test;
    use super::super::{serde::Serde, Occurrence, Skull, WithId};

    type SkullId = <Skull as super::super::Data>::Id;
    type OccurrenceId = <Occurrence as super::super::Data>::Id;

    #[bench]
    fn serialize_skull(bench: &mut test::Bencher) {
        let skull = Skull {
            name: String::from("xnamex"),
            color: String::from("xcolorx"),
            icon: String::from("xiconx"),
            unit_price: 0.1,
            limit: None,
        };

        bench.iter(|| {
            let mut buffer = vec![];
            let mut serder = Serde::new(&mut buffer);

            (0..100)
                .map(|i| SkullId::new(i, skull.clone()))
                .for_each(|s| {
                    serde::Serialize::serialize(&s, &mut serder).unwrap();
                });
        });
    }

    #[bench]
    fn serialize_occurrence(bench: &mut test::Bencher) {
        let occurrence = Occurrence {
            skull: 0,
            amount: 1.2,
            millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        };

        bench.iter(|| {
            let mut buffer = vec![];
            let mut serder = super::super::serde::Serde::new(&mut buffer);

            (0..100)
                .map(|i| OccurrenceId::new(i, occurrence.clone()))
                .for_each(|s| {
                    serde::Serialize::serialize(&s, &mut serder).unwrap();
                });
        });
    }
}

mod csv {
    extern crate test;
    use super::super::{Occurrence, Skull, WithId};

    type SkullId = <Skull as super::super::Data>::Id;
    type OccurrenceId = <Occurrence as super::super::Data>::Id;

    #[bench]
    fn serialize_skull(bench: &mut test::Bencher) {
        let skull = SkullId {
            id: 0,
            name: String::from("xnamex"),
            color: String::from("xcolorx"),
            icon: String::from("xiconx"),
            unit_price: 0.1,
            limit: None,
        };

        bench.iter(|| {
            let buffer = vec![];

            let mut writer = csv::WriterBuilder::new()
                .delimiter(b'\t')
                .has_headers(false)
                .from_writer(buffer);

            (0..100)
                .map(|i| {
                    let mut s = skull.clone();
                    s.id = i;
                    s
                })
                .for_each(|s| writer.serialize(s).unwrap());
        });
    }

    #[bench]
    fn deserialize_skull(bench: &mut test::Bencher) {
        let data = (0..100)
            .map(|i| format!("xnamex\txcolorx\txiconx\t1.2\t{i}\n"))
            .map(|s| s.into_bytes())
            .flatten()
            .collect::<Vec<_>>();

        bench.iter(|| {
            let data = data.clone();

            let mut reader = csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .has_headers(false)
                .from_reader(data.as_slice());

            reader
                .deserialize::<Skull>()
                .enumerate()
                .for_each(|(i, s)| {
                    let s = s.unwrap();
                    assert_eq!(s.name, "xnamex");
                    assert_eq!(s.color, "xcolorx");
                    assert_eq!(s.icon, "xiconx");
                    assert_eq!(s.unit_price, 1.2);
                    assert_eq!(s.limit.unwrap(), i as f32);
                })
        });
    }

    #[bench]
    fn serialize_occurrence(bench: &mut test::Bencher) {
        let occurrence = Occurrence {
            skull: 0,
            amount: 1.2,
            millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        };

        bench.iter(|| {
            let buffer = vec![];

            let mut writer = csv::WriterBuilder::new()
                .delimiter(b'\t')
                .has_headers(false)
                .from_writer(buffer);

            (0..100)
                .map(|i| OccurrenceId::new(i, occurrence.clone()))
                .for_each(|s| writer.serialize(s).unwrap());
        });
    }

    #[bench]
    fn deserialize_occurrence(bench: &mut test::Bencher) {
        let data = (0..100)
            .map(|_| String::from("0\t1.2\t4\n"))
            .map(|s| s.into_bytes())
            .flatten()
            .collect::<Vec<_>>();

        bench.iter(|| {
            let data = data.clone();

            let mut reader = csv::ReaderBuilder::new()
                .delimiter(b'\t')
                .has_headers(false)
                .from_reader(data.as_slice());

            reader
                .deserialize::<Occurrence>()
                .enumerate()
                .for_each(|(_, s)| {
                    let s = s.unwrap();
                    assert_eq!(s.skull, 0);
                    assert_eq!(s.amount, 1.2);
                    assert_eq!(s.millis, 4);
                })
        });
    }
}

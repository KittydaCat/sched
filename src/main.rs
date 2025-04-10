use std::{collections::HashMap, usize};

#[derive(Clone, Debug)]
struct Event<'a> {
    name: &'a str,
    constraints: Vec<Constraint<'a>>,
}

#[derive(Clone, Debug)]
enum Constraint<'a> {
    Sequential(Vec<&'a str>),
    Concurrent(Vec<&'a str>),
    At(Time, Vec<&'a str>),
    In(Room, Vec<&'a str>),
    NonConcurrent(Vec<&'a str>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Room {
    room_num: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Time {
    time_slot: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Placement {
    room: Room,
    time: Time,
}

fn eval_constraints<'a>(constraints: &[Constraint<'a>], event_names: &[&'a str]) -> Vec<Event<'a>> {
    event_names
        .iter()
        .map(|name| {
            let mut event = Event {
                name,
                constraints: Vec::new(),
            };

            for constraint in constraints {
                match constraint {
                    Constraint::Sequential(x)
                    | Constraint::Concurrent(x)
                    | Constraint::NonConcurrent(x)
                    | Constraint::At(_, x)
                    | Constraint::In(_, x) => {
                        if x.iter().find(|found| *found == name).is_some() {
                            event.constraints.push(constraint.clone());
                        }
                    }
                }
            }

            event
        })
        .collect()
}

fn create_schedule<'a>(
    raw_events: &[Event<'a>],
    room_count: usize,
    time_slots: usize,
) -> Box<[Box<[Option<&'a str>]>]> {
    let mut schedule: Box<[Box<[Option<&str>]>]> =
        vec![vec![None; room_count].into_boxed_slice(); time_slots].into_boxed_slice();

    let mut event_map: HashMap<&str, (&Event, Option<Placement>)> = raw_events
        .iter()
        .map(|event| (event.name, (event, None)))
        .collect();

    let mut placements = Vec::<Placement>::new();

    let mut curr_event = None::<&Event>;

    let mut index = 0;

    let mut iteration = 0;

    loop {
        iteration = 1 + dbg!(iteration);

        if dbg!(placements.len()) == raw_events.len() {
            dbg!("we done", placements);

            return schedule;
        }

        while curr_event.is_none() {
            let mut sorts = event_map.iter().collect::<Vec<_>>();

            sorts.sort_by(|x, y| (x.0).cmp(y.0));

            let pull = sorts[index];

            index = (index + 1) % raw_events.len();

            // dbg!(&pull);

            if pull.1.1.is_none() {
                curr_event = Some(pull.1.0);
            }

            // dbg!(curr_event);
        }

        dbg!(&curr_event.unwrap().name);
        assert!(curr_event.is_some());

        let mut time = None::<Time>;
        let mut room = None::<Room>;

        let mut banned_times = Vec::new();
        let mut banned_rooms = Vec::new();

        let mut rewind = false;

        for constraint in &curr_event.as_ref().unwrap().constraints {
            //curr_event.take().unwrap().constraints.iter() {

            match constraint {
                Constraint::Sequential(items)
                | Constraint::Concurrent(items)
                | Constraint::NonConcurrent(items) => {
                    for item in items {
                        if let (event, Some(placement)) = event_map.get(item).unwrap() {
                            match constraint {
                                Constraint::Sequential(items) => {
                                    let placed_pos =
                                        items.iter().position(|e| *e == event.name).unwrap();

                                    let placing_pos = items
                                        .iter()
                                        .position(|e| *e == curr_event.as_ref().unwrap().name)
                                        .unwrap();

                                    dbg!(placing_pos, placed_pos);

                                    let seq_time = (placement.time.time_slot + placing_pos)
                                        .checked_sub(placed_pos);

                                    if seq_time.is_none() {
                                        rewind = true;
                                        break;
                                    }

                                    if seq_time.unwrap() >= time_slots {
                                        rewind = true;
                                        break;
                                    }

                                    if time.is_none() && room.is_none() {
                                        time = Some(Time {
                                            time_slot: seq_time.unwrap(),
                                        });
                                        room = Some(placement.room.clone());
                                    } else {
                                        todo!()
                                    }
                                }
                                Constraint::Concurrent(_) => {
                                    if room.is_none() {
                                        room = Some(placement.room.clone());
                                    } else {
                                        todo!();
                                    }
                                }
                                Constraint::NonConcurrent(_) => {
                                    banned_times.push(placement.time.clone());
                                }
                                Constraint::At(_, _) | Constraint::In(_, _) => unreachable!(),
                            }
                        }
                    }
                }
                Constraint::In(req_room, _items) => {
                    if room.is_none() {
                        room = Some(req_room.clone())
                    } else if room.as_ref().unwrap() == req_room {
                        // we good
                    } else {
                        rewind = true;
                        break;
                    }
                }
                Constraint::At(req_time, _items) => {
                    if time.is_none() {
                        time = Some(req_time.clone())
                    } else if time.as_ref().unwrap() == req_time {
                        // we good
                    } else {
                        rewind = true;
                        break;
                    }
                }
            }
        }

        if let Some(x) = &room {
            rewind = rewind || banned_rooms.contains(x);
        }

        if let Some(x) = &time {
            rewind = rewind || banned_times.contains(x);
        }

        // dbg!(&time, &room, &banned_rooms, &banned_times, rewind);
        // TODO make sure that banned and reqired are not united
        if rewind {
            dbg!("rewind");
            let x = placements.pop().unwrap();
            let event_name = schedule[x.time.time_slot][x.room.room_num].take().unwrap();

            let y = event_map.get_mut(event_name).unwrap();

            y.1 = None;

            continue;
        }

        assert!(!rewind);

        // let mut try_time = time;
        // let mut try_room = room;
        let mut time_index = 0;
        let mut room_index = 0;

        dbg!("loop");
        let placement = loop {
            // while time.is_none() && room.is_none() {

            let try_time = time.clone().unwrap_or(Time {
                time_slot: time_index,
            });
            dbg!(&time);
            let try_room = room.clone().unwrap_or(Room {
                room_num: room_index,
            });

            if banned_times.contains(&try_time) {
                time_index = (time_index + 1) % time_slots;
                continue;
            }
            if banned_rooms.contains(&try_room) {
                room_index = (room_index + 1) % room_count;
                continue;
            }

            dbg!(&try_time, &try_room);
            if schedule[try_time.time_slot][try_room.room_num].is_some() {
                time_index = (time_index + 1) % time_slots;
                room_index = (room_index + 1) % room_count;
                continue;
            }

            break dbg!(Placement {
                room: try_room,
                time: try_time,
            });
        };

        dbg!(&curr_event);

        schedule[placement.time.time_slot][placement.room.room_num] =
            Some(curr_event.unwrap().name);

        placements.push(placement.clone());

        event_map.get_mut(curr_event.unwrap().name).unwrap().1 = Some(placement);

        curr_event = None;
    }
}

fn main() {
    let contraints = vec![
        Constraint::Sequential(vec!["AHL1", "AHL2"]),
        Constraint::Sequential(vec!["BHL1", "BHL2"]),
        Constraint::Sequential(vec!["CHL1", "CHL2"]),
        Constraint::NonConcurrent(vec!["AHL1", "AHL2", "AArt"]),
        Constraint::NonConcurrent(vec!["BHL1", "BHL2", "BDrama"]),
        Constraint::NonConcurrent(vec!["CHL1", "CHL2", "CDrama"]),
        Constraint::In(Room { room_num: 1 }, vec!["BDrama", "CDrama"]),
    ];

    let event_names = vec![
        "AHL1", "BHL1", "CHL1", "AHL2", "BHL2", "CHL2", "AArt", "BDrama", "CDrama",
    ];

    let events = eval_constraints(&contraints, &event_names);

    dbg!(create_schedule(&events, 3, 4));
}

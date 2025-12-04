use std::collections::HashMap;

use crate::vanject::Vanject;

#[derive(Debug)]
pub struct World {
    pub id: u8,
    pub y_size: i16,
    pub vanjects: HashMap<i32, Vanject>,
}

impl World {
    pub fn new(id: u8, y_size: i16) -> Self {
        Self {
            id,
            y_size,
            vanjects: HashMap::new(),
        }
    }

    // pub fn get_player_inventory<'b>(
    //     &'a self,
    //     player: &'b Player,
    // ) -> impl Iterator<Item = &'a Vanject> {
    //     let player_bind_id = player.bind.as_ref().and_then(|b| Some(b.id));
    //     self.vanjects.values().filter(move |v| {
    //         player_bind_id.is_some() && player_bind_id.unwrap() as i32 == v.get_station()
    //     })
    // }

    // pub fn add_vanject(&mut self, vanject: Vanject) {
    //     let vanject_id = vanject.id;
    //     self.vanjects.insert(vanject.id, vanject);
    //     println!("World: the vanject `{}` has been added to world #{}", vanject_id, self.id);
    // }
}

use std::collections::BTreeMap;
use std::rc::Rc;

pub trait MapId<ToType, OldId, NewId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> ToType;
}

impl<FromType: MapId<ToType, OldId, NewId>, ToType, OldId, NewId: Ord>
    MapId<BTreeMap<NewId, ToType>, OldId, NewId> for BTreeMap<OldId, FromType>
{
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> BTreeMap<NewId, ToType> {
        self.iter().map(|(k, v)| (map(k), v.map_id(map))).collect()
    }
}

impl<FromType: MapId<ToType, OldId, NewId>, ToType, OldId, NewId> MapId<Rc<ToType>, OldId, NewId>
    for Rc<FromType>
{
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Rc<ToType> {
        Rc::new((**self).map_id(map))
    }
}

impl<FromType: MapId<ToType, OldId, NewId>, ToType, OldId, NewId> MapId<Vec<ToType>, OldId, NewId>
    for Vec<FromType>
{
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Vec<ToType> {
        self.iter().map(|x| x.map_id(map)).collect()
    }
}

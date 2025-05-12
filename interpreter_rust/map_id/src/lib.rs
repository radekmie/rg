use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

pub trait MapId<ToType, OldId, NewId> {
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> ToType;
}

impl<OldId, NewId> MapId<Self, OldId, NewId> for bool {
    fn map_id(&self, _map: &mut impl FnMut(&OldId) -> NewId) -> Self {
        *self
    }
}

impl<OldId, NewId> MapId<Self, OldId, NewId> for isize {
    fn map_id(&self, _map: &mut impl FnMut(&OldId) -> NewId) -> Self {
        *self
    }
}

impl<OldId, NewId> MapId<Self, OldId, NewId> for usize {
    fn map_id(&self, _map: &mut impl FnMut(&OldId) -> NewId) -> Self {
        *self
    }
}

impl<FromType: MapId<ToType, OldId, NewId>, ToType, OldId, NewId> MapId<Arc<ToType>, OldId, NewId>
    for Arc<FromType>
{
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Arc<ToType> {
        Arc::new((**self).map_id(map))
    }
}

impl<FromType: MapId<ToType, OldId, NewId>, ToType, OldId, NewId: Ord>
    MapId<BTreeMap<NewId, ToType>, OldId, NewId> for BTreeMap<OldId, FromType>
{
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> BTreeMap<NewId, ToType> {
        self.iter().map(|(k, v)| (map(k), v.map_id(map))).collect()
    }
}

impl<FromType: MapId<ToType, OldId, NewId>, ToType, OldId, NewId>
    MapId<Option<ToType>, OldId, NewId> for Option<FromType>
{
    fn map_id(&self, map: &mut impl FnMut(&OldId) -> NewId) -> Option<ToType> {
        self.as_ref().map(|x| x.map_id(map))
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

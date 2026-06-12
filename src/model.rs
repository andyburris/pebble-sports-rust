use alloc::ffi::CString;

#[derive(Clone, Copy)]
pub enum LeagueIcon {
    Baseball = 1,
    Basketball = 2,
    Football = 3,
    Hockey = 4,
    Soccer = 5,
    Other = 6,
}
impl TryFrom<i32> for LeagueIcon {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            1 => Ok(LeagueIcon::Baseball),
            2 => Ok(LeagueIcon::Basketball),
            3 => Ok(LeagueIcon::Football),
            4 => Ok(LeagueIcon::Hockey),
            5 => Ok(LeagueIcon::Soccer),
            6 => Ok(LeagueIcon::Other),
            _ => Err(()), // Returns an error for invalid numbers
        }
    }
}


pub struct League {
    pub id: CString,
    pub name: CString,
    pub icon: LeagueIcon,
}
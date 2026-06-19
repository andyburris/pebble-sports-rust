use alloc::ffi::CString;

#[derive(Clone, Copy)]
pub enum Sport {
    Other = 0,
    Baseball = 1,
    Basketball = 2,
    Football = 3,
    Hockey = 4,
    Soccer = 5,
}
impl TryFrom<i32> for Sport {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Sport::Other),
            1 => Ok(Sport::Baseball),
            2 => Ok(Sport::Basketball),
            3 => Ok(Sport::Football),
            4 => Ok(Sport::Hockey),
            5 => Ok(Sport::Soccer),
            _ => Err(()), // Returns an error for invalid numbers
        }
    }
}

#[derive(Clone)]
pub struct League {
    pub id: i32,
    pub name: CString,
    pub icon: Sport,
}

#[derive(Clone)]
pub struct Game {
    pub id: i32,
    pub league: League,
    pub timestamp: i32,
    pub state: GameState,

    pub home_team: TeamState,
    pub away_team: TeamState,
}

#[derive(Clone)] 
pub enum GameState {
    Scheduled,
    Final,
    Active {
        time: CString,
        details: CString,
    },
}

#[derive(Clone)]
pub struct TeamState {
    pub team: Team,
    pub score: CString,
    pub posession: bool,
}

#[derive(Clone)]
pub struct Team {
    pub id: i32,
    pub name: CString,
    pub record: CString,
}
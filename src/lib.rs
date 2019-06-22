#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::insert_into;
use std::env;
use dotenv::dotenv;
use diesel::prelude::*;

mod models;
mod schema;

use crate::models::*;
use crate::schema::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}



pub fn connect_db() -> SqliteConnection {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&db_url).expect("Error connecting to database")
}

//////////////////////////////
// USER
//////////////////////////////

pub fn add_user(id: i32, name: String, conn: &SqliteConnection) {
    let user = Users {
        id,
        name,
        nb_coq: 1000
    };
    insert_into(users::dsl::users).values(user).execute(conn).expect("Failed to add user");
}

pub fn user_exists(id: i32, conn: &SqliteConnection) -> bool {

    let result = users::dsl::users.filter(users::dsl::id.eq(id)).first::<Users>(conn);

    if let Err(_notfound) = result {
        false
    } else {
        true
    }
}

pub fn get_users_bet_color(black: String, white: String, color: String, conn: &SqliteConnection) -> Option<Vec<Users>> {
    match diesel::dsl::sql_query(
        format!("SELECT * FROM users WHERE id IN (SELECT user_id FROM bets WHERE black = \"{}\" AND white = \"{}\" AND color = \'{}\')", black, white, color))
        .load(conn) {

        Ok(users) => Some(users),
        _ => None,
    }
}

fn set_coq_to_user(id: i32, nb_coq: i32, conn: &SqliteConnection) {
    diesel::update(users::dsl::users.find(id)).set(users::dsl::nb_coq.eq(nb_coq)).execute(conn).expect("Failed to update nb_coq");
}

pub fn get_coq_of_user(id: i32, conn: &SqliteConnection) -> i32 {
    match users::dsl::users.select(users::dsl::nb_coq).filter(users::dsl::id.eq(id)).first::<i32>(conn) {
        Ok(nb_coq) => nb_coq,
        Err(_) => -1,
    }
}

///////////////////////////////////
// BETS
///////////////////////////////////

fn add_bet(user_id: i32, black: String, white: String, bet: i32, color: String, conn: &SqliteConnection) {
    let bet = Bets {
        user_id,
        black,
        white,
        bet,
        color,
    };
    insert_into(bets::dsl::bets).values(bet).execute(conn).expect("failed to insert bet");
}

pub fn create_bet(user_id: i32, black: String, white: String, bet: i32, new_coq: i32, color: String, conn: &SqliteConnection) {
    conn.transaction::<_,diesel::result::Error,_>(|| {
        set_coq_to_user(user_id, new_coq, conn);
        add_bet(user_id, black, white, bet, color, conn);

        Ok(())
    }).expect("Could not create bet");
}

pub fn get_bet(user_id: i32, black: String, white: String, conn: &SqliteConnection) -> Option<Bets>{
    // apparently this is not a tuple so idk. wtf
    // let fkin_tuple = (bets::dsl::user_id.eq(user_id), bets::dsl::black.eq(black), bets::dsl::white.eq(white));
    match bets::dsl::bets
        .filter(bets::dsl::user_id.eq(user_id))
        .filter(bets::dsl::black.eq(black))
        .filter(bets::dsl::white.eq(white))
        .first::<Bets>(conn) {

        Ok(bet) => Some(bet),
        _ => None,
    }
}

#[allow(dead_code)]
fn get_bets_of_game(black: String, white: String, conn: &SqliteConnection) -> Option<Vec<Bets>> {
    match bets::dsl::bets
        .filter(bets::dsl::black.eq(black))
        .filter(bets::dsl::white.eq(white))
        .load::<Bets>(conn) {

        Ok(bets) => Some(bets),
        _ => None,
    }
}

#[allow(dead_code)]
fn get_bets_of_game_color(black: String, white: String, color: String, conn: &SqliteConnection) -> Option<Vec<Bets>> {
    match bets::dsl::bets
        .filter(bets::dsl::black.eq(black))
        .filter(bets::dsl::white.eq(white))
        .filter(bets::dsl::color.eq(color))
        .load::<Bets>(conn) {

        Ok(bets) => Some(bets),
        _ => None,
    }
}

/// bet must have same primary key as previous bet (user_id, black and white attributes)
pub fn update_bet(bet: Bets, conn: &SqliteConnection) {
    diesel::update(bets::dsl::bets.find((bet.user_id, bet.black.clone(), bet.white.clone())))
        .set(bet)
        .execute(conn)
        .expect("Could not update bet");
}

pub fn remove_bets_of_game(black: String, white: String, conn: &SqliteConnection) {
    diesel::delete(bets::dsl::bets
        .filter(bets::dsl::black.eq(black.clone()))
        .filter(bets::dsl::white.eq(white.clone())))
        .execute(conn).expect(&format!("Could not delete bets of game: {} vs {}", black, white));
}

#[cfg(test)]
fn reset_database(conn: &SqliteConnection) {
    diesel::delete(bets::dsl::bets).execute(conn);
    diesel::delete(game::dsl::game).execute(conn);
    diesel::delete(users::dsl::users).execute(conn);
}

#[test]
fn test_get_users_bet_color() {
    let conn = connect_db();
    reset_database(&conn);

    add_user(0, "Romain Fecher".to_string(), &conn);
    add_bet(0, "gne".to_string(), "gne".to_string(), 42, "white".to_string(), &conn);

    let expected_users = vec![Users {
        id: 0,
        name: "Romain Fecher".to_string(),
        nb_coq: 1000,
    }];

    assert_eq!(get_users_bet_color("gne".to_string(), "gne".to_string(), "white".to_string(), &conn).unwrap(), expected_users);
    reset_database(&conn);
}

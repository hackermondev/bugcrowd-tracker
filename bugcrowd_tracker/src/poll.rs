pub mod hall_of_fame {
    use bugcrowd_api::{client::BugcrowdApi, models::Hero};
    use log::{debug, trace};
    use tokio::sync::mpsc::Sender;

    use crate::store::HallOfFameStore;

    pub struct Poller {
        pub bugcrowd: BugcrowdApi,
        pub store: HallOfFameStore,
        pub program_handle: String,
        pub channel: Sender<Event>,
    }

    #[derive(Debug)]
    pub enum Event {
        HeroAdded(Hero),
        HeroUpdated(Hero, Hero),
        HeroRemoved(Hero),
    }

    impl Poller {
        pub async fn poll(&mut self) -> Result<(), anyhow::Error> {
            debug!("polling hall of fame");

            let hall_of_fame = self.bugcrowd.hall_of_fame(&self.program_handle).await?;
            let hall_of_fame: Vec<Hero> = hall_of_fame
                .into_iter()
                .filter(|hero| hero.username != "Private user")
                .collect();
            trace!("got heros: {hall_of_fame:?}");

            let updated_hof = hall_of_fame.clone();
            let mut saved_hof = self.store.heros().await?;

            for hero in updated_hof {
                let old_hero = saved_hof.iter().position(|h| h.username == hero.username);
                if old_hero.is_none() {
                    self.channel.send(Event::HeroAdded(hero)).await?;
                    continue;
                }

                let old_hero = old_hero.unwrap();
                let old_hero = saved_hof.remove(old_hero);
                if hero.points != old_hero.points {
                    debug!("updated {old_hero:?} -> {hero:?}");
                    self.channel
                        .send(Event::HeroUpdated(old_hero, hero))
                        .await?;
                }
            }

            for removed_hero in saved_hof {
                debug!("removed {removed_hero:?}");
                self.channel.send(Event::HeroRemoved(removed_hero)).await?;
            }

            self.store.save_heros(hall_of_fame).await?;
            Ok(())
        }
    }
}

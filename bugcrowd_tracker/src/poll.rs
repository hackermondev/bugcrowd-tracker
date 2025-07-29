pub mod hall_of_fame {
    use bugcrowd_api::{
        client::BugcrowdApi,
        models::{ANONYMOUS_HERO_NAME, Hero},
    };
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
            let hall_of_fame: Vec<Hero> = hall_of_fame.into_iter().collect();
            trace!("got heros: {hall_of_fame:?}");

            let updated_hof = hall_of_fame.clone();
            let mut saved_hof = self.store.heros().await?;

            for hero in updated_hof {
                let anonymous = hero.username == ANONYMOUS_HERO_NAME;
                let old_hero = saved_hof.iter().position(|h| h.username == hero.username);

                if old_hero.is_none() {
                    self.channel.send(Event::HeroAdded(hero)).await?;
                    continue;
                }

                let old_hero = old_hero.unwrap();
                let old_hero = saved_hof.remove(old_hero);
                if hero.points != old_hero.points && !anonymous {
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

pub mod disclosed_reports {
    use bugcrowd_api::{client::BugcrowdApi, models::DisclosedReport};
    use log::debug;
    use tokio::sync::mpsc::Sender;

    use crate::store::DisclosedReportsStore;

    pub struct Poller {
        pub bugcrowd: BugcrowdApi,
        pub store: DisclosedReportsStore,
        pub program_handle: String,
        pub channel: Sender<Event>,
    }

    #[derive(Debug)]
    pub enum Event {
        ReportDisclosed(DisclosedReport),
    }

    impl Poller {
        pub async fn poll(&mut self) -> Result<(), anyhow::Error> {
            debug!("polling disclosed reports");

            let last_disclosed = self.store.last_disclosed_report().await?;
            debug!("last_disclosed: {last_disclosed:?}");

            if last_disclosed.is_none() {
                let report = self
                    .bugcrowd
                    .last_disclosed_report(&self.program_handle)
                    .await?;
                let last_disclosed_report = report.unwrap_or(DisclosedReport {
                    id: String::from("0"),
                    title: String::from("stub"),
                    ..Default::default()
                });
                debug!("saving last disclosed report: {last_disclosed_report:?}");
                self.store
                    .set_last_disclosed_report(&last_disclosed_report)
                    .await?;
                return Ok(());
            }

            let last_disclosed = last_disclosed.unwrap();
            let new_disclosed = self
                .bugcrowd
                .disclosed_reports_after(&self.program_handle, &last_disclosed.id)
                .await?;
            if !new_disclosed.is_empty() {
                let last_disclosed = new_disclosed.first().cloned().unwrap();
                for disclosed in new_disclosed {
                    debug!("new disclosed report: {disclosed:?}");
                    self.channel.send(Event::ReportDisclosed(disclosed)).await?;
                }

                self.store
                    .set_last_disclosed_report(&last_disclosed)
                    .await?;
            }

            Ok(())
        }
    }
}

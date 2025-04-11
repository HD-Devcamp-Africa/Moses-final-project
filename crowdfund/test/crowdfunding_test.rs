#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Env, BigInt, Address, symbol, Token};

    #[test]
    fn test_create_campaign() {
        let env = Env::default();

        // Set up the campaign
        let goal = BigInt::from(1000);
        let deadline = env.timestamp() + 3600; // 1 hour from now
        let token = None; // No token (using native currency)
        let creator = env.invoker();

        // Create the campaign
        let mut campaign = Crowdfunding::create_campaign(env.clone(), goal, deadline, token);

        // Check if the campaign was created correctly
        assert_eq!(campaign.creator, creator);
        assert_eq!(campaign.goal, goal);
        assert_eq!(campaign.deadline, deadline);
        assert_eq!(campaign.total_contributed, BigInt::zero());
        assert_eq!(campaign.status, CampaignStatus::Active);
    }

    #[test]
    fn test_contribute() {
        let env = Env::default();

        // Set up the campaign
        let goal = BigInt::from(1000);
        let deadline = env.timestamp() + 3600; // 1 hour from now
        let token = None; // No token (using native currency)
        let mut campaign = Crowdfunding::create_campaign(env.clone(), goal, deadline, token);

        // Simulate a contribution
        let contributor = Address::from_str("contributor_address").unwrap();
        let amount = BigInt::from(100);

        // Call the contribute method
        Crowdfunding::contribute(env.clone(), &mut campaign, amount);

        // Check if the contribution was recorded
        assert_eq!(campaign.total_contributed, amount);
        assert_eq!(campaign.contributions.get(&contributor), Some(amount));
    }

    #[test]
    fn test_withdraw_funds() {
        let env = Env::default();

        // Set up the campaign
        let goal = BigInt::from(1000);
        let deadline = env.timestamp() + 3600; // 1 hour from now
        let token = None; // No token (using native currency)
        let mut campaign = Crowdfunding::create_campaign(env.clone(), goal, deadline, token);

        // Simulate contributions
        let contributor = Address::from_str("contributor_address").unwrap();
        let amount = BigInt::from(100);
        Crowdfunding::contribute(env.clone(), &mut campaign, amount);

        // Simulate creator withdraw
        let creator = env.invoker();
        Crowdfunding::withdraw(env.clone(), &mut campaign);

        // Assert that funds are withdrawn
        assert_eq!(campaign.total_contributed, BigInt::zero());
        assert_eq!(env.balance_of(&creator), amount); // Assuming balance tracking is set up
    }

    #[test]
    fn test_refund() {
        let env = Env::default();

        // Set up the campaign with goal not reached
        let goal = BigInt::from(1000);
        let deadline = env.timestamp() + 3600; // 1 hour from now
        let token = None; // No token (using native currency)
        let mut campaign = Crowdfunding::create_campaign(env.clone(), goal, deadline, token);

        // Simulate a contribution
        let contributor = Address::from_str("contributor_address").unwrap();
        let amount = BigInt::from(100);
        Crowdfunding::contribute(env.clone(), &mut campaign, amount);

        // Simulate the campaign failing (goal not reached, deadline passed)
        env.advance_time(3601); // Advance 1 second past the deadline
        Crowdfunding::finalize_campaign(env.clone(), &mut campaign);

        // Contributor requests a refund
        Crowdfunding::refund(env.clone(), &mut campaign);

        // Check if refund is successful (no funds left in contract)
        assert_eq!(env.balance_of(&contributor), amount);
    }

    #[test]
    fn test_check_status() {
        let env = Env::default();

        // Set up the campaign with a goal and deadline
        let goal = BigInt::from(1000);
        let deadline = env.timestamp() + 3600; // 1 hour from now
        let token = None; // No token (using native currency)
        let mut campaign = Crowdfunding::create_campaign(env.clone(), goal, deadline, token);

        // Check initial status
        assert_eq!(Crowdfunding::check_status(env.clone(), &campaign), CampaignStatus::Active);

        // Simulate time passing to the deadline
        env.advance_time(3601); // 1 second after the deadline
        Crowdfunding::finalize_campaign(env.clone(), &mut campaign);

        // Check status after deadline
        assert_eq!(Crowdfunding::check_status(env.clone(), &campaign), CampaignStatus::Failed);
    }
}

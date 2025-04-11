#![no_std]

use soroban_sdk::{contractimpl, symbol, Address, Env, I32, Map, Vec, BigInt, Bytes, BytesN, Token};

pub struct Crowdfunding;

#[derive(Clone)]
pub struct Campaign {
    creator: Address,
    goal: BigInt,
    deadline: u64,
    contributions: Map<Address, BigInt>,
    total_contributed: BigInt,
    status: CampaignStatus,
    token: Option<Token>, // Optional token for campaign (if using an ERC-20 token)
}

#[derive(Clone)]
pub enum CampaignStatus {
    Active,
    Successful,
    Failed,
    Expired,
}

#[contractimpl]
impl Crowdfunding {
    // Create a new campaign
    pub fn create_campaign(env: Env, goal: BigInt, deadline: u64, token: Option<Token>) -> Campaign {
        let creator = env.invoker();
        Campaign {
            creator,
            goal,
            deadline,
            contributions: Map::new(&env),
            total_contributed: BigInt::zero(),
            status: CampaignStatus::Active,
            token,
        }
    }

    // Event: Contribution received
    pub fn emit_contribution_received(env: Env, contributor: Address, amount: BigInt) {
        env.emit_event(symbol!("ContributionReceived"), &contributor, &amount);
    }

    // Event: Funds withdrawn by creator
    pub fn emit_funds_withdrawn(env: Env, creator: Address, amount: BigInt) {
        env.emit_event(symbol!("FundsWithdrawn"), &creator, &amount);
    }

    // Event: Refund issued
    pub fn emit_refund_issued(env: Env, contributor: Address, amount: BigInt) {
        env.emit_event(symbol!("RefundIssued"), &contributor, &amount);
    }

    // Contribute to the campaign
    pub fn contribute(env: Env, campaign: &mut Campaign, amount: BigInt) {
        assert!(env.timestamp() < campaign.deadline, "Campaign has ended");
        assert!(amount > BigInt::zero(), "Contribution must be positive");

        let contributor = env.invoker();
        let existing_contribution = campaign.contributions.get(&contributor).unwrap_or(BigInt::zero());

        // Update contribution
        campaign.contributions.insert(&contributor, existing_contribution + amount);
        campaign.total_contributed += amount;

        // Emit ContributionReceived event
        Self::emit_contribution_received(env, contributor, amount);

        // Check if the campaign goal is met
        if campaign.total_contributed >= campaign.goal {
            campaign.status = CampaignStatus::Successful;
        }
    }

    // Withdraw funds by the creator (if campaign is successful)
    pub fn withdraw(env: Env, campaign: &mut Campaign) {
        let creator = env.invoker();
        assert!(creator == campaign.creator, "Only the creator can withdraw");
        assert!(campaign.status == CampaignStatus::Successful, "Campaign goal was not met");

        let amount_to_withdraw = campaign.total_contributed.clone();
        campaign.total_contributed = BigInt::zero();

        // Transfer the funds to the creator (use token if available, else native currency)
        if let Some(token) = &campaign.token {
            token.transfer_from_contract(&creator, amount_to_withdraw.clone());
        } else {
            env.transfer_from_contract(&creator, amount_to_withdraw.clone());
        }

        // Emit FundsWithdrawn event
        Self::emit_funds_withdrawn(env, creator, amount_to_withdraw);
    }

    // Refund contributions (if campaign fails)
    pub fn refund(env: Env, campaign: &mut Campaign) {
        let contributor = env.invoker();
        assert!(campaign.status == CampaignStatus::Failed, "Campaign was successful, no refund available");

        let amount_contributed = campaign.contributions.get(&contributor).unwrap_or(BigInt::zero());
        assert!(amount_contributed > BigInt::zero(), "No contribution found");

        campaign.contributions.insert(&contributor, BigInt::zero());
        campaign.total_contributed -= amount_contributed.clone();

        // Refund the contributor
        if let Some(token) = &campaign.token {
            token.transfer_from_contract(&contributor, amount_contributed.clone());
        } else {
            env.transfer_from_contract(&contributor, amount_contributed.clone());
        }

        // Emit RefundIssued event
        Self::emit_refund_issued(env, contributor, amount_contributed);
    }

    // Check the current status of the campaign
    pub fn check_status(env: Env, campaign: &Campaign) -> CampaignStatus {
        // If the deadline has passed, mark the campaign as expired if not successful
        if env.timestamp() > campaign.deadline {
            if campaign.total_contributed >= campaign.goal {
                campaign.status = CampaignStatus::Successful;
            } else {
                campaign.status = CampaignStatus::Failed;
            }
        }

        campaign.status.clone()
    }

    // Finalize the campaign if the deadline has passed
    pub fn finalize_campaign(env: Env, campaign: &mut Campaign) {
        if env.timestamp() > campaign.deadline {
            if campaign.total_contributed >= campaign.goal {
                campaign.status = CampaignStatus::Successful;
            } else {
                campaign.status = CampaignStatus::Failed;
            }
        } else {
            campaign.status = CampaignStatus::Expired;
        }
    }
}

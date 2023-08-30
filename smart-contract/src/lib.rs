use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, near_bindgen, require,
    serde::{Deserialize, Serialize},
    store::{LookupMap, UnorderedSet, Vector},
    AccountId, BorshStorageKey, CryptoHash, PanicOnDefault,
};

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Users,
    Messages,
    Message,
    Friends,
    FriendOfUser { user_id: AccountId },
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    /// A set of account ids
    pub users: UnorderedSet<AccountId>,
    /// A mapping from chat_id to vector of messages
    ///
    /// Note: chat_id is the hash of `user_id` and `receiver_id`
    pub messages: LookupMap<CryptoHash, Vector<Message>>,
    /// The `friends` mapping is used to store information about user friendships.
    /// It is a two-tier mapping, where the outer mapping associates a user's `AccountId`
    /// with their friend's `AccountId`. The inner mapping stores a boolean value that
    /// indicates whether the users are friends (true) or not (false).
    pub friends: LookupMap<AccountId, LookupMap<AccountId, bool>>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Message {
    pub author: AccountId,
    pub content: String,
    pub created_at_ms: u64,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            users: UnorderedSet::new(StorageKey::Users),
            messages: LookupMap::new(StorageKey::Messages),
            friends: LookupMap::new(StorageKey::Friends),
        }
    }

    pub fn create_account(&mut self) -> bool {
        let user_id = env::predecessor_account_id();
        self.users.insert(user_id)
    }

    pub fn add_friend(&mut self, friend_id: AccountId) {
        let user_id = env::predecessor_account_id();

        require!(
            self.users.contains(&user_id),
            "You must be a user to add a friend."
        );

        require!(
            self.users.contains(&friend_id),
            "Your friend must be a user."
        );

        require!(user_id != friend_id, "You cannot add yourself as friend.");

        let friends = self.friends.entry(user_id.clone()).or_insert_with(|| {
            LookupMap::new(StorageKey::FriendOfUser {
                user_id: user_id.clone(),
            })
        });

        friends.insert(user_id, true);
    }

    pub fn send_message(&mut self, receiver_id: AccountId, message_content: String) -> CryptoHash {
        let user_id = env::predecessor_account_id();

        require!(
            self.users.contains(&user_id),
            "You must be a user to send a message."
        );

        require!(
            self.users.contains(&receiver_id),
            "The receiver must be a user to receive a message."
        );

        let is_valid_friend = self
            .friends
            .get(&user_id)
            .unwrap_or_else(|| env::panic_str("You do not have any friend."))
            .contains_key(&receiver_id);

        require!(
            !is_valid_friend,
            "You are not friends with the given receiver."
        );

        require!(message_content.is_empty(), "The message can not be empty.");

        let chat_id: CryptoHash = self.get_chat_id(user_id.clone(), receiver_id);

        let messages = self
            .messages
            .entry(chat_id)
            .or_insert_with(|| Vector::new(StorageKey::Message));

        let message = Message {
            author: user_id,
            content: message_content,
            created_at_ms: env::block_timestamp_ms(),
        };

        messages.push(message);

        chat_id
    }

    pub fn get_chat_id(&self, user_id: AccountId, receiver_id: AccountId) -> CryptoHash {
        self.calculate_hash(user_id.as_str(), receiver_id.as_str())
    }

    pub fn get_messages(&self, user_id: AccountId, receiver_id: AccountId) -> &Vector<Message> {
        let chat_id: CryptoHash = self.get_chat_id(user_id, receiver_id);

        self.messages
            .get(&chat_id)
            .unwrap_or_else(|| env::panic_str("The user does not have any messages."))

        // let b = *a;
        // todo!()
        // a
    }

    fn calculate_hash(&self, a: &str, b: &str) -> CryptoHash {
        let concatenated_string = format!("{}{}", a, b);

        let value_hash = env::keccak256(concatenated_string.as_bytes());
        let mut res = CryptoHash::default();
        res.copy_from_slice(&value_hash);

        res
    }
}

#[allow(dead_code, unused)]
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::{test_utils::*, testing_env, AccountId, ONE_NEAR};

    fn contract_account() -> AccountId {
        "contract".parse::<AccountId>().unwrap()
    }

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(contract_account())
            .account_balance(15 * ONE_NEAR)
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test() {}
}

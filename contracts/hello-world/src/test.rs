#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype,
    token, Address, Env, String, Vec,
    symbol_short,
};

// ─────────────────────────────────────────────
// DATA TYPES
// ─────────────────────────────────────────────

/// Loại listing: trao đổi hoặc bán
#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ListingType {
    Exchange, // trao đổi sách
    Sale,     // bán lấy token
}

#[contracttype]
#[derive(Clone)]
pub struct Book {
    pub id: u64,
    pub title: String,
    pub subject: String,      // môn học: Toán, Lý, Hóa...
    pub grade: u32,           // lớp: 1-12
    pub owner: Address,
    pub condition: u32,       // 1-5 (5 = mới nhất)
    pub listing_type: ListingType,
    pub price: i128,          // giá tính bằng token (0 nếu là trao đổi)
    pub is_available: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct User {
    pub address: Address,
    pub books_listed: u64,
    pub exchanges_done: u64,
    pub books_sold: u64,
    pub books_bought: u64,
    pub total_earned: i128,   // tổng token đã nhận được
    pub total_spent: i128,    // tổng token đã chi
}

#[contracttype]
pub enum DataKey {
    BookCount,
    Book(u64),
    User(Address),
    TokenContract,            // địa chỉ token contract
    Admin,                    // admin của marketplace
    FeePercent,               // phí giao dịch (vd: 2 = 2%)
    FeeBalance,               // tổng phí đã thu
}

// ─────────────────────────────────────────────
// CONTRACT
// ─────────────────────────────────────────────

#[contract]
pub struct TextbookExchange;

#[contractimpl]
impl TextbookExchange {

    // ── KHỞI TẠO CONTRACT ──────────────────────
    // Gọi 1 lần duy nhất khi deploy
    // token_contract: địa chỉ SEP-41 token dùng làm tiền tệ
    // fee_percent: phí sàn, vd 2 = 2%

    pub fn initialize(
        env: Env,
        admin: Address,
        token_contract: Address,
        fee_percent: u32,
    ) {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract da duoc khoi tao");
        }

        if fee_percent > 10 {
            panic!("Phi toi da la 10%");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::TokenContract, &token_contract);
        env.storage().instance().set(&DataKey::FeePercent, &fee_percent);
        env.storage().instance().set(&DataKey::FeeBalance, &0i128);
    }

    // ── ĐĂNG SÁCH ĐỂ TRAO ĐỔI ──────────────────

    pub fn list_for_exchange(
        env: Env,
        owner: Address,
        title: String,
        subject: String,
        grade: u32,
        condition: u32,
    ) -> u64 {
        owner.require_auth();
        Self::validate_grade_condition(grade, condition);

        let book_id = Self::next_book_id(&env);

        let book = Book {
            id: book_id,
            title,
            subject,
            grade,
            owner: owner.clone(),
            condition,
            listing_type: ListingType::Exchange,
            price: 0,
            is_available: true,
        };

        env.storage().instance().set(&DataKey::Book(book_id), &book);
        env.storage().instance().set(&DataKey::BookCount, &book_id);
        Self::update_user_stats(&env, &owner, 1, 0, 0, 0, 0, 0);

        env.events().publish(
            (symbol_short!("listed"), owner),
            (book_id, symbol_short!("exchange")),
        );

        book_id
    }

    // ── ĐĂNG SÁCH ĐỂ BÁN ──────────────────────
    // price: số token (tính theo đơn vị nhỏ nhất của token, vd stroop)

    pub fn list_for_sale(
        env: Env,
        owner: Address,
        title: String,
        subject: String,
        grade: u32,
        condition: u32,
        price: i128,
    ) -> u64 {
        owner.require_auth();
        Self::validate_grade_condition(grade, condition);

        if price <= 0 {
            panic!("Gia ban phai lon hon 0");
        }

        let book_id = Self::next_book_id(&env);

        let book = Book {
            id: book_id,
            title,
            subject,
            grade,
            owner: owner.clone(),
            condition,
            listing_type: ListingType::Sale,
            price,
            is_available: true,
        };

        env.storage().instance().set(&DataKey::Book(book_id), &book);
        env.storage().instance().set(&DataKey::BookCount, &book_id);
        Self::update_user_stats(&env, &owner, 1, 0, 0, 0, 0, 0);

        env.events().publish(
            (symbol_short!("listed"), owner),
            (book_id, symbol_short!("sale"), price),
        );

        book_id
    }

    // ── MUA SÁCH (thanh toán bằng token) ───────
    // Buyer phải approve contract trước khi gọi hàm này

    pub fn buy_book(env: Env, buyer: Address, book_id: u64) {
        buyer.require_auth();

        let mut book: Book = env
            .storage()
            .instance()
            .get(&DataKey::Book(book_id))
            .expect("Khong tim thay sach");

        if book.listing_type != ListingType::Sale {
            panic!("Sach nay chi de trao doi, khong phai de ban");
        }
        if !book.is_available {
            panic!("Sach nay da duoc ban");
        }
        if book.owner == buyer {
            panic!("Ban khong the mua sach cua chinh minh");
        }

        let token_contract: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenContract)
            .expect("Chua khoi tao token contract");

        let fee_percent: u32 = env
            .storage()
            .instance()
            .get(&DataKey::FeePercent)
            .unwrap_or(0u32);

        let fee_amount = (book.price * fee_percent as i128) / 100;
        let seller_amount = book.price - fee_amount;

        let token = token::Client::new(&env, &token_contract);

        // Chuyển token từ buyer → seller
        token.transfer(&buyer, &book.owner, &seller_amount);

        // Chuyển phí vào contract
        if fee_amount > 0 {
            let contract_addr = env.current_contract_address();
            token.transfer(&buyer, &contract_addr, &fee_amount);

            // Cộng dồn phí
            let current_fee: i128 = env
                .storage()
                .instance()
                .get(&DataKey::FeeBalance)
                .unwrap_or(0i128);
            env.storage()
                .instance()
                .set(&DataKey::FeeBalance, &(current_fee + fee_amount));
        }

        // Đánh dấu sách đã bán
        book.is_available = false;
        env.storage().instance().set(&DataKey::Book(book_id), &book);

        // Cập nhật stats
        Self::update_user_stats(&env, &book.owner, 0, 0, 1, 0, seller_amount, 0);
        Self::update_user_stats(&env, &buyer, 0, 0, 0, 1, 0, book.price);

        env.events().publish(
            (symbol_short!("sold"), buyer.clone()),
            (book_id, book.price, buyer),
        );
    }

    // ── YÊU CẦU TRAO ĐỔI ───────────────────────

    pub fn request_exchange(env: Env, requester: Address, book_id: u64) {
        requester.require_auth();

        let mut book: Book = env
            .storage()
            .instance()
            .get(&DataKey::Book(book_id))
            .expect("Khong tim thay sach");

        if book.listing_type != ListingType::Exchange {
            panic!("Sach nay de ban, khong phai de trao doi. Dung buy_book()");
        }
        if !book.is_available {
            panic!("Sach nay da duoc trao doi");
        }
        if book.owner == requester {
            panic!("Ban khong the trao doi sach cua chinh minh");
        }

        book.is_available = false;
        env.storage().instance().set(&DataKey::Book(book_id), &book);

        Self::update_user_stats(&env, &book.owner, 0, 1, 0, 0, 0, 0);
        Self::update_user_stats(&env, &requester, 0, 1, 0, 0, 0, 0);

        env.events().publish(
            (symbol_short!("exchange"), requester),
            book_id,
        );
    }

    // ── CẬP NHẬT GIÁ BÁN ──────────────────────

    pub fn update_price(env: Env, owner: Address, book_id: u64, new_price: i128) {
        owner.require_auth();

        let mut book: Book = env
            .storage()
            .instance()
            .get(&DataKey::Book(book_id))
            .expect("Khong tim thay sach");

        if book.owner != owner {
            panic!("Ban khong co quyen sua gia sach nay");
        }
        if book.listing_type != ListingType::Sale {
            panic!("Sach nay khong phai de ban");
        }
        if !book.is_available {
            panic!("Sach nay da duoc ban");
        }
        if new_price <= 0 {
            panic!("Gia phai lon hon 0");
        }

        book.price = new_price;
        env.storage().instance().set(&DataKey::Book(book_id), &book);

        env.events().publish(
            (symbol_short!("repriced"), owner),
            (book_id, new_price),
        );
    }

    // ── HỦY ĐĂNG SÁCH ──────────────────────────

    pub fn unlist_book(env: Env, owner: Address, book_id: u64) {
        owner.require_auth();

        let mut book: Book = env
            .storage()
            .instance()
            .get(&DataKey::Book(book_id))
            .expect("Khong tim thay sach");

        if book.owner != owner {
            panic!("Ban khong co quyen huy sach nay");
        }
        if !book.is_available {
            panic!("Sach nay khong con available");
        }

        book.is_available = false;
        env.storage().instance().set(&DataKey::Book(book_id), &book);

        env.events().publish(
            (symbol_short!("unlisted"), owner),
            book_id,
        );
    }

    // ── ADMIN RÚT PHÍ ──────────────────────────

    pub fn withdraw_fees(env: Env) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("Chua khoi tao");

        admin.require_auth();

        let fee_balance: i128 = env
            .storage()
            .instance()
            .get(&DataKey::FeeBalance)
            .unwrap_or(0i128);

        if fee_balance == 0 {
            panic!("Khong co phi de rut");
        }

        let token_contract: Address = env
            .storage()
            .instance()
            .get(&DataKey::TokenContract)
            .expect("Chua khoi tao token contract");

        let token = token::Client::new(&env, &token_contract);
        let contract_addr = env.current_contract_address();
        token.transfer(&contract_addr, &admin, &fee_balance);

        env.storage().instance().set(&DataKey::FeeBalance, &0i128);
    }

    // ── QUERY ───────────────────────────────────

    pub fn get_book(env: Env, book_id: u64) -> Book {
        env.storage()
            .instance()
            .get(&DataKey::Book(book_id))
            .expect("Khong tim thay sach")
    }

    pub fn search_by_grade(env: Env, grade: u32) -> Vec<Book> {
        Self::filter_books(&env, |b| b.grade == grade && b.is_available)
    }

    pub fn search_by_subject(env: Env, subject: String) -> Vec<Book> {
        Self::filter_books(&env, |b| b.subject == subject && b.is_available)
    }

    pub fn search_for_sale(env: Env) -> Vec<Book> {
        Self::filter_books(&env, |b| {
            b.listing_type == ListingType::Sale && b.is_available
        })
    }

    pub fn search_for_exchange(env: Env) -> Vec<Book> {
        Self::filter_books(&env, |b| {
            b.listing_type == ListingType::Exchange && b.is_available
        })
    }

    pub fn get_user(env: Env, address: Address) -> User {
        env.storage()
            .instance()
            .get(&DataKey::User(address.clone()))
            .unwrap_or(User {
                address,
                books_listed: 0,
                exchanges_done: 0,
                books_sold: 0,
                books_bought: 0,
                total_earned: 0,
                total_spent: 0,
            })
    }

    pub fn total_books(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::BookCount)
            .unwrap_or(0u64)
    }

    pub fn get_fee_balance(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::FeeBalance)
            .unwrap_or(0i128)
    }

    pub fn get_token_contract(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::TokenContract)
            .expect("Chua khoi tao")
    }

    // ─────────────────────────────────────────
    // INTERNAL HELPERS
    // ─────────────────────────────────────────

    fn next_book_id(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::BookCount)
            .unwrap_or(0u64)
            + 1
    }

    fn validate_grade_condition(grade: u32, condition: u32) {
        if grade < 1 || grade > 12 {
            panic!("grade phai tu 1 den 12");
        }
        if condition < 1 || condition > 5 {
            panic!("condition phai tu 1 den 5");
        }
    }

    fn filter_books<F>(env: &Env, predicate: F) -> Vec<Book>
    where
        F: Fn(&Book) -> bool,
    {
        let total: u64 = env
            .storage()
            .instance()
            .get(&DataKey::BookCount)
            .unwrap_or(0u64);

        let mut results: Vec<Book> = Vec::new(env);

        for id in 1..=total {
            if let Some(book) = env
                .storage()
                .instance()
                .get::<DataKey, Book>(&DataKey::Book(id))
            {
                if predicate(&book) {
                    results.push_back(book);
                }
            }
        }

        results
    }

    fn update_user_stats(
        env: &Env,
        address: &Address,
        listed_delta: u64,
        exchange_delta: u64,
        sold_delta: u64,
        bought_delta: u64,
        earned_delta: i128,
        spent_delta: i128,
    ) {
        let mut user: User = env
            .storage()
            .instance()
            .get(&DataKey::User(address.clone()))
            .unwrap_or(User {
                address: address.clone(),
                books_listed: 0,
                exchanges_done: 0,
                books_sold: 0,
                books_bought: 0,
                total_earned: 0,
                total_spent: 0,
            });

        user.books_listed   += listed_delta;
        user.exchanges_done += exchange_delta;
        user.books_sold     += sold_delta;
        user.books_bought   += bought_delta;
        user.total_earned   += earned_delta;
        user.total_spent    += spent_delta;

        env.storage()
            .instance()
            .set(&DataKey::User(address.clone()), &user);
    }
}

// ─────────────────────────────────────────────
// TESTS
// ─────────────────────────────────────────────

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{token, Env};
    use soroban_sdk::token::StellarAssetClient;

    fn setup_token(env: &Env, admin: &Address) -> Address {
        let token_id = env.register_stellar_asset_contract_v2(admin.clone());
        let stellar_asset = StellarAssetClient::new(env, &token_id.address());
        stellar_asset.mint(admin, &1_000_000_000);
        token_id.address()
    }

    fn setup_contract(env: &Env) -> (TextbookExchangeClient, Address, Address) {
        let admin = Address::generate(env);
        let token_contract = setup_token(env, &admin);
        let contract_id = env.register_contract(None, TextbookExchange);
        let client = TextbookExchangeClient::new(env, &contract_id);
        client.initialize(&admin, &token_contract, &2u32); // phí 2%
        (client, admin, token_contract)
    }

    #[test]
    fn test_list_exchange_and_search() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, _) = setup_contract(&env);

        let user = Address::generate(&env);
        let id = client.list_for_exchange(
            &user,
            &String::from_str(&env, "Toan 10 - Canh Dieu"),
            &String::from_str(&env, "Toan"),
            &10u32,
            &4u32,
        );
        assert_eq!(id, 1);

        let books = client.search_by_grade(&10u32);
        assert_eq!(books.len(), 1);

        let exchange_books = client.search_for_exchange();
        assert_eq!(exchange_books.len(), 1);

        let sale_books = client.search_for_sale();
        assert_eq!(sale_books.len(), 0);
    }

    #[test]
    fn test_buy_book_with_token() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, token_contract) = setup_contract(&env);

        let seller = Address::generate(&env);
        let buyer  = Address::generate(&env);

        // Mint token cho buyer
        let stellar_asset = token::StellarAssetClient::new(&env, &token_contract);
        stellar_asset.mint(&buyer, &10_000);

        // Seller đăng sách giá 1000 token
        let book_id = client.list_for_sale(
            &seller,
            &String::from_str(&env, "Ly 12 - Ket noi"),
            &String::from_str(&env, "Ly"),
            &12u32,
            &3u32,
            &1000i128,
        );

        // Buyer mua
        client.buy_book(&buyer, &book_id);

        // Kiểm tra sách đã bán
        let book = client.get_book(&book_id);
        assert_eq!(book.is_available, false);

        // Kiểm tra token: seller nhận 980 (1000 - 2% phí), contract giữ 20
        let token_client = token::Client::new(&env, &token_contract);
        assert_eq!(token_client.balance(&seller), 980);
        assert_eq!(client.get_fee_balance(), 20);

        // Stats
        let seller_stats = client.get_user(&seller);
        assert_eq!(seller_stats.books_sold, 1);
        assert_eq!(seller_stats.total_earned, 980);

        let buyer_stats = client.get_user(&buyer);
        assert_eq!(buyer_stats.books_bought, 1);
        assert_eq!(buyer_stats.total_spent, 1000);
    }

    #[test]
    fn test_update_price() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, _, _) = setup_contract(&env);

        let seller = Address::generate(&env);
        let book_id = client.list_for_sale(
            &seller,
            &String::from_str(&env, "Hoa 11"),
            &String::from_str(&env, "Hoa"),
            &11u32,
            &5u32,
            &500i128,
        );

        client.update_price(&seller, &book_id, &800i128);
        let book = client.get_book(&book_id);
        assert_eq!(book.price, 800);
    }

    #[test]
    fn test_withdraw_fees() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin, token_contract) = setup_contract(&env);

        let seller = Address::generate(&env);
        let buyer  = Address::generate(&env);

        let stellar_asset = token::StellarAssetClient::new(&env, &token_contract);
        stellar_asset.mint(&buyer, &10_000);

        let book_id = client.list_for_sale(
            &seller,
            &String::from_str(&env, "Van 9"),
            &String::from_str(&env, "Van"),
            &9u32,
            &2u32,
            &1000i128,
        );

        client.buy_book(&buyer, &book_id);
        assert_eq!(client.get_fee_balance(), 20);

        client.withdraw_fees();
        assert_eq!(client.get_fee_balance(), 0);
    }
}

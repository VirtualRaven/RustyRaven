use std::collections::{BTreeMap, BTreeSet, HashMap};

use stripe::{
    generated::{billing::tax_rate, core::tax_code}, CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession, CreateCheckoutSessionLineItems, CreateCheckoutSessionLineItemsPriceData, CreateCheckoutSessionLineItemsPriceDataProductData, CreateCheckoutSessionLineItemsPriceDataTaxBehavior, CreateCheckoutSessionPhoneNumberCollection, CreateCheckoutSessionShippingAddressCollection, CreateCheckoutSessionShippingAddressCollectionAllowedCountries, CreateCustomer, CreatePrice, CreateProduct, CreateTaxRate, Currency, Customer, Expandable, IdOrCreate, ListTaxRates, Price, Product, TaxRate, TaxRateId
};
use tracing::info;


const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAJOR_VERSION: &str = env!("CARGO_PKG_VERSION_MAJOR");
const NAME: &str = env!("CARGO_PKG_NAME");
pub const CANCLE_PATH: &str = "/order/avbruten";
pub const SUCCESS_PATH: &str = "/order/klar";

fn client() -> ::stripe::Client
{
    let secret_key = dotenvy::var("STRIPE_API_KEY").expect("Missing STRIPE_API_KEY");
    let client = Client::new(secret_key)
    .with_app_info(
        NAME.into(),
        Some(VERSION.into()),
        None
    );

    client
}





use once_cell::sync::OnceCell;
static TAX_RATES: OnceCell<BTreeMap<u8,TaxRateId>> = OnceCell::new();
static SITE_URL: OnceCell<String> = OnceCell::new();


pub async fn init() -> Result<(),crate::PaymentError>
{
    SITE_URL.set( dotenvy::var("WEBSITE_URL").unwrap() ).unwrap();
    create_swedish_tax_rates().await
}

 async fn create_swedish_tax_rates() -> Result<(),crate::PaymentError>
{
    let predefined_tax_rates: BTreeSet<_> =  [25u8,12,6,0].into_iter().collect();
    let client = client();

    let params = ListTaxRates {
        active: Some(true),
        ..Default::default()
    };


    let exsisting_rates = TaxRate::list(&client, &params).await?;
    let list = exsisting_rates.paginate(params);
    let mut stream = list.stream(&client);
    use futures_util::TryStreamExt;


    let description_string = format!("tax-{}-{}",NAME,MAJOR_VERSION);


    let mut tax_rate_names = BTreeMap::<u8,TaxRateId>::new();

    while let Some(existing_tax_rate) = stream.try_next().await?
    {

        let swedish =  existing_tax_rate.country.unwrap_or_default() == "SE";
        let from_app = existing_tax_rate.description.unwrap_or_default() == description_string;
        let rate : u8 = existing_tax_rate.percentage as u8;
        let id = existing_tax_rate.id;
        if  swedish && from_app && predefined_tax_rates.contains(&rate)
        {
            info!("Found STRIPE tax rate {}",id);
            tax_rate_names.insert(rate,id);
        }
    }

    for predefined_tax_rate in predefined_tax_rates 
    {
        if !tax_rate_names.contains_key(&predefined_tax_rate)
        {
            info!("Creating STRIPE tax rate {}", predefined_tax_rate);
            let display_name = format!("Moms {}%", predefined_tax_rate);
            let mut tax_rate = CreateTaxRate::new(&display_name , predefined_tax_rate as f64);
            tax_rate.inclusive = true;
            tax_rate.country = Some("SE");
            tax_rate.description = Some(&description_string);
            tax_rate.active = Some(true);
            tax_rate.tax_type = Some( stripe::TaxRateTaxType::Vat );
            let created_tax_rate =  TaxRate::create(&client, tax_rate).await?;
            tax_rate_names.insert(predefined_tax_rate, created_tax_rate.id);
        }

    }

    TAX_RATES.set(tax_rate_names).unwrap();

    Ok(())
}


pub async fn checkout(uuid: String) -> Result<String,crate::PaymentError> {
    let url = SITE_URL.get().unwrap();
    let client = client();


    let items = sjf_db::checkout::get_order(&uuid).await?;


    let items = items.into_iter().map(|item|{

        let item_tax_rate = item.tax_rate as u8;
        let tax_rate_id = TAX_RATES.get().unwrap().get(&item_tax_rate).ok_or(crate::PaymentError::InvalidTaxRate(item_tax_rate))?;
        let tax_rate_id = String::from(tax_rate_id.as_str());
        let image_urls = item.image_path.map(|i|  vec![format!("{}{}",url,i)]);

        Ok::<CreateCheckoutSessionLineItems, crate::PaymentError>(CreateCheckoutSessionLineItems {
            quantity: Some(item.ordered_quantity.into()),
            tax_rates: Some(vec![tax_rate_id]),
            price_data: Some(
                CreateCheckoutSessionLineItemsPriceData {
                    currency:  Currency::SEK,
                    product_data: Some(
                        CreateCheckoutSessionLineItemsPriceDataProductData {
                            description: None,
                            images: image_urls  ,
                            name: item.name,
                            tax_code: None,
                            ..Default::default()
                        }
                    ),
                    tax_behavior: Some(CreateCheckoutSessionLineItemsPriceDataTaxBehavior::Inclusive),
                    unit_amount: Some((100*item.price).into()),
                    ..Default::default()
                }
            ),
            ..Default::default()
        })

    })
    .collect::<Result<Vec<_>,_>>()?;


    let checkout_session = {
        let mut params = CreateCheckoutSession::new();
        
        let cancel_url = format!("{}{}/{}",url,CANCLE_PATH,uuid);
        let success_url = format!("{}{}/{}",url,SUCCESS_PATH,uuid);

        params.cancel_url = Some(&cancel_url);
        params.success_url = Some(&success_url);
        params.client_reference_id = Some(uuid.as_ref());
        params.metadata = Some(
            {
                let mut map: HashMap<String,String> = HashMap::new();
                map.insert("reservation".into(), uuid.clone());
                map.insert("app".into(), NAME.into());
                map.insert("app-version".into(), VERSION.into());
                map
            }
        );
        params.phone_number_collection = Some(
            CreateCheckoutSessionPhoneNumberCollection {
                enabled: true
            }
        );
        params.shipping_address_collection = Some(
            CreateCheckoutSessionShippingAddressCollection {
                allowed_countries: vec![CreateCheckoutSessionShippingAddressCollectionAllowedCountries::Se ]
            }
        );

        params.billing_address_collection = Some(
            stripe::CheckoutSessionBillingAddressCollection::Auto
        );

        use std::time::{SystemTime, UNIX_EPOCH};
        let expiry = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()+30*60;
        params.expires_at = Some(expiry as i64);

        params.mode = Some(CheckoutSessionMode::Payment);
        params.line_items = Some(items);

        CheckoutSession::create(&client, params).await?
    };


    Ok(checkout_session.url.ok_or(crate::PaymentError::NoUrl )? )
}
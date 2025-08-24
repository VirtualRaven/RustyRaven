use std::collections::{BTreeMap, BTreeSet, HashMap};

use stripe::{
    CheckoutSession, CheckoutSessionMode, Client, CreateCheckoutSession,
    CreateCheckoutSessionCustomFields, CreateCheckoutSessionCustomFieldsDropdown,
    CreateCheckoutSessionCustomFieldsDropdownOptions, CreateCheckoutSessionCustomFieldsLabel,
    CreateCheckoutSessionCustomFieldsType, CreateCheckoutSessionLineItems,
    CreateCheckoutSessionLineItemsPriceData, CreateCheckoutSessionLineItemsPriceDataProductData,
    CreateCheckoutSessionLineItemsPriceDataTaxBehavior, CreateCheckoutSessionPaymentIntentData,
    CreateCheckoutSessionPhoneNumberCollection, CreateCheckoutSessionShippingAddressCollection,
    CreateCheckoutSessionShippingAddressCollectionAllowedCountries,
    CreateCheckoutSessionShippingOptions, CreateCheckoutSessionShippingOptionsShippingRateData,
    CreateCheckoutSessionShippingOptionsShippingRateDataDeliveryEstimate,
    CreateCheckoutSessionShippingOptionsShippingRateDataDeliveryEstimateMaximum,
    CreateCheckoutSessionShippingOptionsShippingRateDataDeliveryEstimateMaximumUnit,
    CreateCheckoutSessionShippingOptionsShippingRateDataDeliveryEstimateMinimum,
    CreateCheckoutSessionShippingOptionsShippingRateDataDeliveryEstimateMinimumUnit,
    CreateCheckoutSessionShippingOptionsShippingRateDataFixedAmount,
    CreateCheckoutSessionShippingOptionsShippingRateDataTaxBehavior,
    CreateCheckoutSessionShippingOptionsShippingRateDataType, CreateCustomer, CreatePrice,
    CreateProduct, CreateTaxRate, Currency, Customer, Expandable, IdOrCreate, ListTaxRates,
    OrderItem, Price, Product, TaxRate, TaxRateId,
    generated::{billing::tax_rate, core::tax_code},
};
use tracing::info;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const MAJOR_VERSION: &str = env!("CARGO_PKG_VERSION_MAJOR");
const NAME: &str = env!("CARGO_PKG_NAME");

fn client() -> ::stripe::Client {
    let secret_key = dotenvy::var("STRIPE_API_KEY").expect("Missing STRIPE_API_KEY");
    let client = Client::new(secret_key).with_app_info(NAME.into(), Some(VERSION.into()), None);

    client
}

//struct ShippingOption {
//    maximum: u32,
//    minimum: u32,
//    ammount: u32,
//    display_name: String,
//}

//impl ShippingOption {
//    fn to_stripe(&self) -> CreateCheckoutSessionShippingOptions {
//        CreateCheckoutSessionShippingOptions {
//            shipping_rate_data: Some(
//                CreateCheckoutSessionShippingOptionsShippingRateData {
//                    delivery_estimate: Some(
//                            CreateCheckoutSessionShippingOptionsShippingRateDataDeliveryEstimate
//                            {
//                                maximum: Some(
//                                    CreateCheckoutSessionShippingOptionsShippingRateDataDeliveryEstimateMaximum
//                                    {
//                                        value: self.maximum as i64,
//                                        unit: CreateCheckoutSessionShippingOptionsShippingRateDataDeliveryEstimateMaximumUnit::BusinessDay
//                                    }
//                                ),
//                                minimum: Some(
//                                    CreateCheckoutSessionShippingOptionsShippingRateDataDeliveryEstimateMinimum
//                                    {
//                                        value: self.minimum as i64,
//                                        unit: CreateCheckoutSessionShippingOptionsShippingRateDataDeliveryEstimateMinimumUnit::BusinessDay
//                                    }
//                                )
//                            }
//                        ),
//                    display_name: self.display_name.clone(),
//                    fixed_amount: Some(
//                        CreateCheckoutSessionShippingOptionsShippingRateDataFixedAmount {
//                            amount: (self.ammount*100) as i64,
//                            currency: Currency::SEK,
//                            ..Default::default()
//                        }
//                    ),
//                    tax_behavior: Some(CreateCheckoutSessionShippingOptionsShippingRateDataTaxBehavior::Inclusive),
//                    tax_code: Some("txcd_92010001".into()),
//                    type_ : Some(CreateCheckoutSessionShippingOptionsShippingRateDataType::FixedAmount) ,
//                    ..Default::default()
//                },
//
//            ),
//            shipping_rate: None,
//        }
//    }
//}

use once_cell::sync::OnceCell;
static TAX_RATES: OnceCell<BTreeMap<u8, TaxRateId>> = OnceCell::new();
static SITE_URL: OnceCell<String> = OnceCell::new();

pub async fn init() -> Result<(), crate::PaymentError> {
    SITE_URL.set(dotenvy::var("WEBSITE_URL").unwrap()).unwrap();
    create_swedish_tax_rates().await
}

async fn create_swedish_tax_rates() -> Result<(), crate::PaymentError> {
    let predefined_tax_rates: BTreeSet<_> = [25u8, 12, 6, 0].into_iter().collect();
    let client = client();

    let params = ListTaxRates {
        active: Some(true),
        ..Default::default()
    };

    let exsisting_rates = TaxRate::list(&client, &params).await?;
    let list = exsisting_rates.paginate(params);
    let mut stream = list.stream(&client);
    use futures_util::TryStreamExt;

    let description_string = format!("tax-{}-{}", NAME, MAJOR_VERSION);

    let mut tax_rate_names = BTreeMap::<u8, TaxRateId>::new();

    while let Some(existing_tax_rate) = stream.try_next().await? {
        let swedish = existing_tax_rate.country.unwrap_or_default() == "SE";
        let from_app = existing_tax_rate.description.unwrap_or_default() == description_string;
        let rate: u8 = existing_tax_rate.percentage as u8;
        let id = existing_tax_rate.id;
        if swedish && from_app && predefined_tax_rates.contains(&rate) {
            info!("Found STRIPE tax rate {}", id);
            tax_rate_names.insert(rate, id);
        }
    }

    for predefined_tax_rate in predefined_tax_rates {
        if !tax_rate_names.contains_key(&predefined_tax_rate) {
            info!("Creating STRIPE tax rate {}", predefined_tax_rate);
            let display_name = format!("Moms {}%", predefined_tax_rate);
            let mut tax_rate = CreateTaxRate::new(&display_name, predefined_tax_rate as f64);
            tax_rate.inclusive = true;
            tax_rate.country = Some("SE");
            tax_rate.description = Some(&description_string);
            tax_rate.active = Some(true);
            tax_rate.tax_type = Some(stripe::TaxRateTaxType::Vat);
            let created_tax_rate = TaxRate::create(&client, tax_rate).await?;
            tax_rate_names.insert(predefined_tax_rate, created_tax_rate.id);
        }
    }

    TAX_RATES.set(tax_rate_names).unwrap();

    Ok(())
}

pub async fn checkout(uuid: String) -> Result<String, crate::PaymentError> {
    let url = SITE_URL.get().unwrap();
    let client = client();

    let items = {
        let mut items = sjf_db::checkout::get_order(&uuid).await?;
        let shipping_price = {
            let total_order_quantity = items
                .iter()
                .map(|i| i.ordered_quantity)
                .fold(0, |acc, i| acc + i);
            let total_order_price = items
                .iter()
                .map(|i| i.price * i.ordered_quantity)
                .fold(0, |acc, i| acc + i);

            let initial_price = match total_order_quantity {
                0..=2 => 89,
                3..=4 => 99,
                5.. => 129,
            };

            if total_order_price > 999 {
                0
            } else {
                initial_price
            }
        };

        items.push(sjf_db::checkout::OrderItem {
            product_id: 0,
            image_path: None,
            name: "Frakt".into(),
            price: shipping_price,
            ordered_quantity: 1,
            tax_rate: 25,
        });

        items
    };

    //let shipping_options = vec![
    //    ShippingOption {
    //        maximum: 5,
    //        minimum: 2,
    //        ammount: shipping_price,
    //        display_name: "PostNord".into(),
    //    }
    //    .to_stripe(),
    //    ShippingOption {
    //        maximum: 5,
    //        minimum: 2,
    //        ammount: shipping_price,
    //        display_name: "Schenker".into(),
    //    }
    //    .to_stripe(),
    //];

    let metadata = {
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert("reservation".into(), uuid.clone());
        map.insert("app".into(), NAME.into());
        map.insert("app-version".into(), VERSION.into());
        map
    };

    let items = items
        .into_iter()
        .map(|item| {
            let item_tax_rate = item.tax_rate as u8;
            let tax_rate_id = TAX_RATES
                .get()
                .unwrap()
                .get(&item_tax_rate)
                .ok_or(crate::PaymentError::InvalidTaxRate(item_tax_rate))?;
            let tax_rate_id = String::from(tax_rate_id.as_str());
            let image_urls = item.image_path.map(|i| vec![format!("{}{}", url, i)]);

            Ok::<CreateCheckoutSessionLineItems, crate::PaymentError>(
                CreateCheckoutSessionLineItems {
                    quantity: Some(item.ordered_quantity.into()),
                    tax_rates: Some(vec![tax_rate_id]),
                    price_data: Some(CreateCheckoutSessionLineItemsPriceData {
                        currency: Currency::SEK,
                        product_data: Some(CreateCheckoutSessionLineItemsPriceDataProductData {
                            description: None,
                            images: image_urls,
                            name: item.name,
                            tax_code: None,
                            metadata: Some({
                                let mut map = metadata.clone();
                                map.insert(
                                    "article-number".into(),
                                    format!("artikel-{}", item.product_id),
                                );
                                map
                            }),
                            ..Default::default()
                        }),
                        tax_behavior: Some(
                            CreateCheckoutSessionLineItemsPriceDataTaxBehavior::Inclusive,
                        ),
                        unit_amount: Some((100 * item.price).into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let metadata = {
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert("reservation".into(), uuid.clone());
        map.insert("app".into(), NAME.into());
        map.insert("app-version".into(), VERSION.into());
        map
    };

    let checkout_session = {
        let mut params = CreateCheckoutSession::new();

        let cancel_url = format!("{}{}/{}", url, sjf_api::payment::CANCLE_PATH, uuid);
        let success_url = format!("{}{}/{}", url, sjf_api::payment::SUCCESS_PATH, uuid);

        //params.shipping_options = Some(shipping_options);
        params.cancel_url = Some(&cancel_url);
        params.success_url = Some(&success_url);
        params.client_reference_id = Some(uuid.as_ref());
        params.customer_creation = Some(stripe::CheckoutSessionCustomerCreation::Always);
        params.payment_intent_data = Some(CreateCheckoutSessionPaymentIntentData {
            metadata: Some(metadata.clone()),
            ..Default::default()
        });
        params.metadata = Some(metadata);
        params.phone_number_collection =
            Some(CreateCheckoutSessionPhoneNumberCollection { enabled: true });
        params.shipping_address_collection = Some(CreateCheckoutSessionShippingAddressCollection {
            allowed_countries: vec![
                CreateCheckoutSessionShippingAddressCollectionAllowedCountries::Se,
            ],
        });

        params.custom_fields = Some(vec![CreateCheckoutSessionCustomFields {
            dropdown: Some(CreateCheckoutSessionCustomFieldsDropdown {
                options: vec![
                    CreateCheckoutSessionCustomFieldsDropdownOptions {
                        label: "PostNord".into(),
                        value: "PostNord".into(),
                    },
                    CreateCheckoutSessionCustomFieldsDropdownOptions {
                        label: "Schenker".into(),
                        value: "Schenker".into(),
                    },
                ],
            }),
            label: CreateCheckoutSessionCustomFieldsLabel {
                custom: "Frakt alternativ".into(),
                type_: stripe::CreateCheckoutSessionCustomFieldsLabelType::Custom,
            },
            key: "selected_shipping_option".into(),
            optional: Some(false),
            type_: CreateCheckoutSessionCustomFieldsType::Dropdown,
            ..Default::default()
        }]);
        params.billing_address_collection =
            Some(stripe::CheckoutSessionBillingAddressCollection::Auto);

        use std::time::{SystemTime, UNIX_EPOCH};
        let expiry = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 30 * 60;
        params.expires_at = Some(expiry as i64);

        params.mode = Some(CheckoutSessionMode::Payment);
        params.line_items = Some(items);

        CheckoutSession::create(&client, params).await?
    };

    Ok(checkout_session.url.ok_or(crate::PaymentError::NoUrl)?)
}

# Cost-Aware Routing Demo Guide

## Overview

This guide demonstrates the custom **Cost-Aware Routing** strategy implemented in Hyperswitch.

Additionally, I have attached the Postman collection containing all the API requests used in the demo. It can be imported directly to execute the complete Cost-Aware Routing flow without manually creating the requests.

The routing configuration intentionally prioritizes connectors in a more expensive order:

```
Priority Routing
----------------
1. Paypal (35 bps)
2. Stripe (25 bps)
```

During payment execution, the custom Cost-Aware Routing strategy reorders the eligible connectors based on their configured processing cost.

Expected execution order:

```
Eligibility
-----------
Paypal
   ↓
Stripe

Cost-Aware Routing
------------------
Stripe
   ↓
Paypal
```

This proves that the routing decision is modified **after eligibility analysis but before connector execution**.

---

# Prerequisites

Before starting, ensure the following:

- Hyperswitch is running locally on:

```text
http://localhost:8080
```

- Docker services are up and healthy.
- `curl` is installed.
- `jq` is installed.
- Server log level is **DEBUG** (default in `config/development.toml`).
- A Stripe **Test Secret Key** is available.

Example:

```text
sk_test_xxxxxxxxxxxxxxxxxxxxxxxxx
```

> **Note**
>
> Paypal connector creation does **not** validate credentials during registration.
> Placeholder credentials are sufficient because authentication happens only during payment execution.
>
> Stripe requires a valid test key if you want the payment to succeed.

---

# Demo Flow

The demo consists of the following steps:

1. Create Merchant
2. Create API Key
3. Register Paypal Connector
4. Register Stripe Connector
5. Create Priority Routing
6. Activate Routing
7. Submit Payment
8. Verify Routing Logs

---

# Step 1 — Create Merchant

Create a demo merchant.

```bash
curl -s -X POST http://localhost:8080/accounts \
  -H "api-key: test_admin" \
  -H "Content-Type: application/json" \
  -d '{
    "merchant_id": "cost_demo_merchant",
    "merchant_name": "Cost Demo",
    "merchant_details": {
      "primary_email": "demo@example.com"
    },
    "sub_merchants_enabled": false,
    "primary_business_details": [
      {
        "country": "US",
        "business": "default"
      }
    ]
  }' | jq '{ merchant_id, default_profile }'
```

### Expected Response

```json
{
  "merchant_id": "cost_demo_merchant",
  "default_profile": "pro_xxxxxxxxx"
}
```

### Save

Copy the value of:

```text
default_profile
```

Use it as:

```text
<PROFILE_ID>
```

in Step 5.

---

# Step 2 — Create API Key

Create an API key for the merchant.

```bash
curl -s -X POST http://localhost:8080/api_keys/cost_demo_merchant \
  -H "api-key: test_admin" \
  -H "Content-Type: application/json" \
  -d '{
    "name":"demo-key",
    "expiration":"2069-01-01T00:00:00.000Z"
  }' | jq -r '.api_key'
```

### Expected Response

```text
snd_xxxxxxxxxxxxxxxxxxxxxxxxx
```

### Save

Copy the returned API key.

Replace:

```text
<API_KEY>
```

in every subsequent request.

Example:

```http
api-key: <API_KEY>
```

---

# Step 3 — Register Paypal Connector

This connector intentionally has a **higher processing cost (35 bps)**.

Placeholder credentials are sufficient.

```bash
curl -s -X POST http://localhost:8080/account/cost_demo_merchant/connectors \
  -H "api-key: <API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{
    "connector_type":"payment_processor",
    "connector_name":"paypal",
    "business_country":"US",
    "business_label":"default",
    "connector_account_details":{
        "auth_type":"BodyKey",
        "api_key":"placeholder_client_id",
        "key1":"placeholder_secret"
    },
    "test_mode":true,
    "disabled":false,
    "payment_methods_enabled":[{
        "payment_method":"card",
        "payment_method_types":[{
            "payment_method_type":"credit",
            "card_networks":["Visa","Mastercard"],
            "minimum_amount":1,
            "maximum_amount":68607706,
            "recurring_enabled":true,
            "installment_payment_enabled":true
        }]
    }]
  }' | jq -r '.merchant_connector_id'
```

### Expected Response

```text
mca_xxxxxxxxx
```

### Save

Copy the returned Merchant Connector ID.

Replace:

```text
<PAYPAL_MCA>
```

in Step 5.

---

# Step 4 — Register Stripe Connector

This connector intentionally has a **lower processing cost (25 bps)**.

Replace:

```text
sk_test_YOUR_STRIPE_KEY
```

with your Stripe Test Secret Key.

```bash
curl -s -X POST http://localhost:8080/account/cost_demo_merchant/connectors \
  -H "api-key: <API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{
    "connector_type":"payment_processor",
    "connector_name":"stripe",
    "business_country":"US",
    "business_label":"default",
    "connector_label":"stripe_demo",
    "connector_account_details":{
        "auth_type":"HeaderKey",
        "api_key":"sk_test_YOUR_STRIPE_KEY"
    },
    "test_mode":true,
    "disabled":false,
    "payment_methods_enabled":[{
        "payment_method":"card",
        "payment_method_types":[{
            "payment_method_type":"credit",
            "card_networks":["Visa","Mastercard"],
            "minimum_amount":1,
            "maximum_amount":68607706,
            "recurring_enabled":true,
            "installment_payment_enabled":true
        }]
    }]
  }' | jq -r '.merchant_connector_id'
```

### Expected Response

```text
mca_xxxxxxxxx
```

### Save

Copy the returned Merchant Connector ID.

Replace:

```text
<STRIPE_MCA>
```

in Step 5.

---

# Step 5 — Create Priority Routing

This routing configuration intentionally places **Paypal before Stripe**.

The Cost-Aware Routing strategy will reorder this during execution.

Replace the placeholders:

- `<PROFILE_ID>`
- `<PAYPAL_MCA>`
- `<STRIPE_MCA>`

```bash
curl -s -X POST http://localhost:8080/routing \
  -H "api-key: <API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{
    "name":"cost-demo-priority",
    "description":"Paypal first; Cost-Aware Routing promotes Stripe",
    "profile_id":"<PROFILE_ID>",
    "algorithm":{
        "type":"priority",
        "data":[
            {
                "connector":"paypal",
                "merchant_connector_id":"<PAYPAL_MCA>"
            },
            {
                "connector":"stripe",
                "merchant_connector_id":"<STRIPE_MCA>"
            }
        ]
    }
  }' | jq -r '.id'
```

### Expected Response

```text
routing_xxxxxxxxx
```

### Save

Copy the Routing ID.

Replace:

```text
<ROUTING_ID>
```

in Step 6.

---

# Step 6 — Activate Routing

Activate the routing algorithm.

```bash
curl -s -X POST http://localhost:8080/routing/<ROUTING_ID>/activate \
  -H "api-key: <API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{}'
```

### Expected Result

```text
HTTP 200 OK
```

The routing algorithm is now active.

---

# Step 7 — Submit Payment

This request triggers the complete routing pipeline, including the custom Cost-Aware Routing logic.

```bash
curl -s -X POST http://localhost:8080/payments \
  -H "api-key: <API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{
    "amount":6540,
    "currency":"USD",
    "confirm":true,
    "capture_method":"automatic",
    "amount_to_capture":6540,
    "customer_id":"demo_customer",
    "email":"guest@example.com",
    "name":"John Doe",
    "description":"Cost-Aware Routing Demo",
    "authentication_type":"no_three_ds",
    "return_url":"https://duck.com",
    "payment_method":"card",
    "payment_method_type":"credit",
    "payment_method_data":{
        "card":{
            "card_number":"4242424242424242",
            "card_exp_month":"01",
            "card_exp_year":"30",
            "card_holder_name":"John Doe",
            "card_cvc":"123"
        }
    },
    "billing":{
        "address":{
            "line1":"1467",
            "line2":"Harrison Street",
            "city":"San Francisco",
            "state":"California",
            "zip":"94122",
            "country":"US",
            "first_name":"John",
            "last_name":"Doe"
        }
    }
  }' | jq '{ payment_id, status, connector }'
```

### Expected Response

```json
{
  "payment_id":"pay_xxxxxxxxx",
  "status":"succeeded",
  "connector":"stripe"
}
```

> If invalid Stripe credentials are used, the payment may fail after routing.
> This does **not** affect the routing demonstration.
> The routing decision can still be verified from the server logs.

---

# Expected Routing Logs

After submitting the payment, observe the server logs.

The following DEBUG entries should appear.

## Before Cost-Aware Routing

```text
DEBUG euclid:
connectors after eligibility = {paypal, stripe}
```

This reflects the configured Priority Routing.

---

## After Cost-Aware Routing

```text
DEBUG euclid:
connectors after cost-aware = {stripe, paypal}
```

This confirms that the custom routing strategy reordered the eligible connectors according to processing cost.

---

# Verification Checklist

- [ ] Merchant created successfully
- [ ] API key generated
- [ ] Paypal connector registered
- [ ] Stripe connector registered
- [ ] Priority routing created
- [ ] Routing activated
- [ ] Payment request executed
- [ ] Payment selected **Stripe** instead of **Paypal**
- [ ] Eligibility log displayed:
  - `paypal → stripe`
- [ ] Cost-Aware Routing log displayed:
  - `stripe → paypal`

---

# Success Criteria

The implementation is considered successful when the following conditions are met:

**Configured Priority Routing**

```text
Paypal
   ↓
Stripe
```

**Actual Execution Order**

```text
Stripe
   ↓
Paypal
```

without modifying the configured routing algorithm.

This demonstrates that the custom Cost-Aware Routing strategy executes **after eligibility analysis** and **before connector execution**, reordering the final connector list based on processing cost.
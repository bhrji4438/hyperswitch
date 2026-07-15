use api_models::routing::{RoutableChoiceKind, RoutableConnectorChoice};
use euclid::enums::RoutableConnectors;

// Costs are illustrative demo values representing relative processing cost in
// basis points. They are not production gateway pricing and should be replaced
// with merchant-specific MDR data before use in a production environment.
fn connector_cost(connector: RoutableConnectors) -> u32 {
    match connector {
        RoutableConnectors::Cybersource => 18,
        RoutableConnectors::Adyen => 20,
        RoutableConnectors::Nmi => 20,
        RoutableConnectors::Checkout => 22,
        RoutableConnectors::Worldpay => 22,
        RoutableConnectors::Stripe => 25,
        RoutableConnectors::Nuvei => 25,
        RoutableConnectors::Bluesnap => 28,
        RoutableConnectors::Braintree => 29,
        RoutableConnectors::Paypal => 35,
        _ => u32::MAX,
    }
}

/// Re-ranks the connector list in ascending order of processing cost.
///
/// Uses a stable sort so that equal-cost connectors preserve the relative
/// ordering established by the prior routing stage (SR ranking, static
/// priority list). Connectors absent from the cost table sort last,
/// preserving their inter-order from the input.
pub fn apply_cost_aware_routing(
    mut connectors: Vec<RoutableConnectorChoice>,
) -> Vec<RoutableConnectorChoice> {
    connectors.sort_by_key(|c| connector_cost(c.connector));
    connectors
}

#[cfg(test)]
mod test {
    use super::*;

    fn make_choice(connector: RoutableConnectors) -> RoutableConnectorChoice {
        RoutableConnectorChoice {
            choice_kind: RoutableChoiceKind::OnlyConnector,
            connector,
            merchant_connector_id: None,
        }
    }

    #[test]
    fn test_apply_cost_aware_routing_orders_by_cost() {
        let input = vec![
            make_choice(RoutableConnectors::Paypal),      // cost 35
            make_choice(RoutableConnectors::Stripe),      // cost 25
            make_choice(RoutableConnectors::Cybersource), // cost 18
        ];
        let result = apply_cost_aware_routing(input);
        assert_eq!(result[0].connector, RoutableConnectors::Cybersource);
        assert_eq!(result[1].connector, RoutableConnectors::Stripe);
        assert_eq!(result[2].connector, RoutableConnectors::Paypal);
    }

    #[test]
    fn test_apply_cost_aware_routing_stable_on_equal_cost() {
        // Stripe and Nuvei both map to cost 25; input order must be preserved.
        let input = vec![
            make_choice(RoutableConnectors::Stripe), // cost 25, position 0
            make_choice(RoutableConnectors::Nuvei),  // cost 25, position 1
        ];
        let result = apply_cost_aware_routing(input);
        assert_eq!(result[0].connector, RoutableConnectors::Stripe);
        assert_eq!(result[1].connector, RoutableConnectors::Nuvei);
    }

    #[test]
    fn test_apply_cost_aware_routing_empty_input() {
        let result = apply_cost_aware_routing(vec![]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_cost_aware_routing_single_connector() {
        let input = vec![make_choice(RoutableConnectors::Stripe)];
        let result = apply_cost_aware_routing(input);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].connector, RoutableConnectors::Stripe);
    }

    #[test]
    fn test_apply_cost_aware_routing_unknown_connector_sorts_last() {
        // Klarna is absent from the cost table and must fall after Adyen.
        let input = vec![
            make_choice(RoutableConnectors::Klarna), // cost u32::MAX
            make_choice(RoutableConnectors::Adyen),  // cost 20
        ];
        let result = apply_cost_aware_routing(input);
        assert_eq!(result[0].connector, RoutableConnectors::Adyen);
        assert_eq!(result[1].connector, RoutableConnectors::Klarna);
    }
}

# Shopify product metafield definitions

Shop: `68c5b4-6a.myshopify.com` (API 2026-01), owner type `PRODUCT`. 68 definitions, read from Shopify on 2026-10-24.
Options come from the `choices` validation; other validations (metaobject definition ids, limits, file types) are in the Other validations column.

| Label | Namespace.key | Type | Options | Other validations | Description |
|---|---|---|---|---|---|
| Color | `custom.color_furniture` | `single_line_text_field` |  |  | Color of published furniture |
| compare.at.price | `custom.compare_at_price` | `single_line_text_field` |  |  |  |
| Condition report | `custom.condition_report` | `multi_line_text_field` |  |  | Simple, customer-facing summary of the item's real damages, drawn from the Furniture Inspection Report. Lists only components that have an actual issue; skips anything marked None or N/A. |
| Condition Report Description | `custom.condition_report_description` | `multi_line_text_field` |  |  |  |
| Condition.Cracks | `custom.condition_cracks` | `single_line_text_field` | N/A; None; Minor; Moderate; Major |  |  |
| Condition.Dents/Chips | `custom.condition_dents_chips` | `single_line_text_field` | N/A; None; Minor; Moderate; Major |  |  |
| Condition.Function | `custom.condition_function` | `single_line_text_field` | N/A; None; Minor; Moderate; Major |  |  |
| Condition.Missing Parts | `custom.condition_missing_parts` | `single_line_text_field` | N/A; None; Minor; Moderate; Major |  |  |
| Condition.Mold | `custom.condition_mold` | `single_line_text_field` | N/A; None; Minor; Moderate; Major |  |  |
| Condition.Odor | `custom.condition_odor` | `single_line_text_field` | N/A; None; Minor; Moderate; Major |  |  |
| Condition.Scratch | `custom.condition_scratch` | `single_line_text_field` | N/A; None; Minor; Moderate; Major |  |  |
| Condition.Stability | `custom.condition_stability` | `single_line_text_field` | N/A; None; Minor; Moderate; Major |  |  |
| Condition.Stains/Discoloration | `custom.condition_stains_discoloration` | `single_line_text_field` | N/A; None; Minor; Moderate; Major |  |  |
| Condition.Upholstery | `custom.condition_upholstery` | `single_line_text_field` | N/A; None; Minor; Moderate; Major |  |  |
| Condition.Veneer Damage | `custom.condition_veneer_damage` | `single_line_text_field` | N/A; None; Minor; Moderate; Major |  |  |
| Condition.Water Damage | `custom.condition_water_damage` | `single_line_text_field` | N/A; None; Minor; Moderate; Major |  |  |
| Contains Key | `custom.contains_key` | `single_line_text_field` | Yes (1 pcs); Yes (2 pcs); No |  |  |
| Country | `custom.country` | `single_line_text_field` | Denmark; Sweden; Germany; Norway; Finland; United Kingdom; Hungary; Italy; Netherland; Other |  | Made in Country?  |
| Damage details | `custom.damage_details` | `list.single_line_text_field` | Surface has been refinished; Completely restored; Partially restored; Light wear consistent with age and use; Patina consistent with age and use; Heavy signs of use; Reupholstered; Requires reupholstery; Rewired and/or new electrical components; Requires rewiring and/or new electrical parts |  |  |
| Damage report | `custom.damage_report` | `list.file_reference` |  | file_type_options=["Image"] |  |
| Depth | `custom.totaldepth` | `single_line_text_field` |  |  | Total depth of furniture |
| Depth Dimension | `custom.depth_dimension` | `dimension` |  |  |  |
| Designer | `custom.designer` | `single_line_text_field` |  |  | Designer of furniture |
| Detailed condition | `custom.detailed_condition` | `single_line_text_field` | Excellent - This vintage piece is in near original condition. It may show minimal traces of use and have slight restorations.; Very Good - This vintage item has no defects, but it may show slight traces of use.; Good - This vintage item may have some wear such as scratches and other signs of ageing; Worn - This vintage item has visible damages, defects, or small missing parts. It may be in need of restoration. |  | Detailed condition |
| Dimension | `custom.dimensionss` | `multi_line_text_field` |  |  | Dimensions for furniture  |
| Etsy description | `custom.etsy_description` | `multi_line_text_field` |  |  |  |
| Extension Dimension | `custom.extension_dimension` | `dimension` |  |  |  |
| Extension Quantity | `custom.extension_quantity` | `single_line_text_field` | 1; 2; 3; 4; 5; 6; 7; 8 |  |  |
| Extension Type | `custom.extension_type` | `single_line_text_field` | Inside Extension; Outside Extension |  | Inside extension: Rectangular/Oval/Round tables that can not live without extension boards.  |
| Extensions Quantity | `custom.extensions_quantity` | `number_integer` |  |  |  |
| Height | `custom.totalheight` | `single_line_text_field` |  |  | Total height of furniture  |
| Height Dimension | `custom.height_dimension` | `dimension` |  |  |  |
| Inspection Report | `custom.inspection_report` | `metaobject_reference` |  | metaobject_definition_id=gid://shopify/MetaobjectDefinition/46710391114 |  |
| Link | `custom.link` | `url` |  |  |  |
| Location | `custom.location` | `single_line_text_field` | Västberga Warehouse; Västberga store |  | Location where item is located  |
| Manufacturer | `custom.manufacturer` | `single_line_text_field` |  |  | Name of manufacturer |
| Mark / Stamp | `custom.mark_stamp` | `single_line_text_field` |  |  |  |
| Price/st | `custom.price_st` | `single_line_text_field` |  |  | Price per piece |
| Quantity | `custom.quantity` | `single_line_text_field` | 1; 2; 3; 4; 5; 6; 7; 8; 9; 10 |  | quantity |
| reserved | `custom.reserved` | `list.single_line_text_field` | Reserved; Non-reserved |  |  |
| Restoration | `custom.restoration` | `rich_text_field` |  |  |  |
| Seatheight | `custom.seatheightchairs` | `single_line_text_field` |  |  | Seat height of chairs/armchairs/sofas |
| Shape | `custom.shape` | `single_line_text_field` |  |  |  |
| Utstallning | `custom.utstallning` | `multi_line_text_field` |  |  |  |
| video and condition | `custom.video_and_condition` | `file_reference` |  | file_type_options=["Image","Video"] | Videos and photos of condition  |
| Weight definition | `custom.weight_definition` | `single_line_text_field` | 1-20 kg; 21-40 kg; 41-60 kg; 61+ kg |  | weight definition |
| Width | `custom.totalwidth` | `single_line_text_field` |  |  | Total width of furniture |
| Width Dimension | `custom.width_dimension` | `dimension` |  |  |  |
| Wood type | `custom.wood_type` | `list.single_line_text_field` | Teak; Dark Teak; Oak; Dark Oak; Santos Rosewood; Mahogany; Walnut; Cherry; Beech; Birch; Elm; Pine; Chrome/Steel  |  | Wood type  |
| Years | `custom.years` | `single_line_text_field` | Early 20th century furniture; 1950-1960s; 1960-1970s; 1970-1980s; 1980-1990s; 1990-2000s; 2000s+ |  | When was it made |
| Item Location | `app.item_location` | `single_line_text_field` | H1; H2; H3; O1; O2; K1; K2; R11; R12; R13; R14; R15; R21; R22; R23; R24; R25; R31; R32; R33; R34; R35; E11; E12; E21; R26; LC0; LC1; LC2; LC3; LC4; LC5; LC6; LC7; LC8; LC9; LC10; LC12; LC13; ZF |  |  |
| Bought together products | `globo--filter--product_recommendation.bought_together_products` | `list.product_reference` |  |  |  |
| Complementary products | `globo--filter--product_recommendation.complementary_products` | `list.product_reference` |  |  |  |
| Related products | `globo--filter--product_recommendation.related_products` | `list.product_reference` |  |  |  |
| Product rating | `reviews.rating` | `rating` |  | scale_min=1.0; scale_max=5.0 | Average rating from customers |
| Product rating count | `reviews.rating_count` | `number_integer` |  | min=0 | Total number of ratings from customers |
| Back type | `shopify.back-type` | `list.metaobject_reference` |  | metaobject_definition_id=gid://shopify/MetaobjectDefinition/19305464138 | Defines the style of back support on seating, such as backless or low back |
| Backrest type | `shopify.backrest-type` | `list.metaobject_reference` |  | metaobject_definition_id=gid://shopify/MetaobjectDefinition/19057410378 | Specifies the material or style of a chair's backrest; e.g. mesh, padded |
| Chair features | `shopify.chair-features` | `list.metaobject_reference` |  | metaobject_definition_id=gid://shopify/MetaobjectDefinition/21934833994 | Highlights special functions of a chair, such as massage function or power reclining |
| Color | `shopify.color-pattern` | `list.metaobject_reference` |  | metaobject_definition_id=gid://shopify/MetaobjectDefinition/19037389130 | Defines the primary color or pattern, such as blue or striped |
| Lumber/Wood type | `shopify.lumber-wood-type` | `list.metaobject_reference` |  | metaobject_definition_id=gid://shopify/MetaobjectDefinition/19305824586 | Identifies the specific type of wood or lumber used in a product, like oak or pine |
| Seat type | `shopify.seat-type` | `list.metaobject_reference` |  | metaobject_definition_id=gid://shopify/MetaobjectDefinition/19057443146 | Describes the design of a chair or bench seat, such as padded or bucket  |
| Upholstery material | `shopify.upholstery-material` | `list.metaobject_reference` |  | metaobject_definition_id=gid://shopify/MetaobjectDefinition/19305496906 | Identifies the fabric used for upholstery, such as leather or velvet |
| Washing method | `shopify.washing-method` | `list.metaobject_reference` |  | metaobject_definition_id=gid://shopify/MetaobjectDefinition/19306152266 | Specifies the recommended cleaning process for a product, e.g. hand washing, machine washing  |
| Complementary products | `shopify--discovery--product_recommendation.complementary_products` | `list.product_reference` |  | list.max=10 | List of complementary products |
| Related products | `shopify--discovery--product_recommendation.related_products` | `list.product_reference` |  | list.max=10 | List of related products |
| Related products settings | `shopify--discovery--product_recommendation.related_products_display` | `single_line_text_field` | ahead; only manual |  | Determines how related products are displayed along with algorithmic product recommendations |
| Search product boosts | `shopify--discovery--product_search_boost.queries` | `list.single_line_text_field` |  | max=100; list.max=10 | List of search queries for which a product gets higher rank in search results |

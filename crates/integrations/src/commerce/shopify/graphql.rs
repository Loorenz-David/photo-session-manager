//! Admin GraphQL documents (hand-written, no codegen). These are the exact documents the P12
//! grounding probe (`examples/shopify_probe.rs`) sent to the test store, so every fixture in
//! `fixtures/` is a request/response pair of the document the adapter sends. API 2026-10 has no
//! `productCreateMedia` / `productDeleteMedia`: media are created with `productUpdate(media:)` and
//! removed from a product with `fileUpdate(referencesToRemove:)` (SV-2, SV-4).

pub const DEFINITIONS: &str = "query Definitions($after: String) { metafieldDefinitions(ownerType: PRODUCT, first: 50, after: $after) { pageInfo { hasNextPage endCursor } nodes { id namespace key name description type { name } validations { name value } access { admin } } } }";

pub const PRODUCT_READ: &str = "query ProductRead($id: ID!) { product(id: $id) { id title productType status tags descriptionHtml variants(first: 5) { nodes { id sku barcodes(first: 5) { nodes { value type } } inventoryItem { sku } } } media(first: 50) { nodes { id status alt mediaContentType ... on MediaImage { image { url } originalSource { url } } } } } }";

pub const VARIANT_SEARCH: &str = "query VariantSearch($query: String!) { productVariants(first: 10, query: $query) { nodes { id sku barcodes(first: 5) { nodes { value } } product { id title productType status featuredMedia { preview { image { url } } } } } } }";
pub const TITLE_SEARCH: &str = "query TitleSearch($query: String!) { products(first: 20, query: $query) { nodes { id title productType featuredMedia { preview { image { url } } } variants(first: 1) { nodes { sku barcodes(first: 5) { nodes { value } } inventoryItem { sku } } } } } }";

pub const METAFIELDS_READ: &str = "query MetafieldsRead($id: ID!, $after: String) { product(id: $id) { metafields(first: 100, after: $after) { pageInfo { hasNextPage endCursor } nodes { namespace key type value } } } }";
pub const CONDITION_METAFIELD_READ: &str = "query ConditionMetafieldRead($id: ID!, $namespace: String!, $key: String!) { product(id: $id) { metafield(namespace: $namespace, key: $key) { value } } }";

pub const METAFIELDS_SET: &str = "mutation MetafieldsSet($metafields: [MetafieldsSetInput!]!) { metafieldsSet(metafields: $metafields) { metafields { namespace key type value } userErrors { field message code elementIndex } } }";

pub const STAGED_UPLOADS_CREATE: &str = "mutation StagedUploadsCreate($input: [StagedUploadInput!]!) { stagedUploadsCreate(input: $input) { stagedTargets { url resourceUrl parameters { name value } } userErrors { field message } } }";

pub const PRODUCT_CREATE_MEDIA: &str = "mutation ProductCreateMedia($product: ProductUpdateInput!, $media: [CreateMediaInput!]) { productUpdate(product: $product, media: $media) { product { id newest: media(first: 1, reverse: true) { nodes { id status alt mediaContentType } } all: media(first: 50) { nodes { id status alt } } } userErrors { field message } } }";

/// P64: several media in one `productUpdate`; the `$count` newest media are the created ones.
pub const PRODUCT_CREATE_MEDIA_MANY: &str = "mutation ProductCreateMediaMany($product: ProductUpdateInput!, $media: [CreateMediaInput!], $count: Int!) { productUpdate(product: $product, media: $media) { product { id newest: media(first: $count, reverse: true) { nodes { id status alt mediaContentType } } } userErrors { field message } } }";

/// P62 (10F Rules, M41): the product's gallery media and its condition-photos field's files,
/// each with id and image URL, in one read.
pub const MEDIA_READBACK: &str = "query MediaReadback($id: ID!, $namespace: String!, $key: String!) { product(id: $id) { id media(first: 250) { nodes { id status ... on MediaImage { image { url } originalSource { url } } } } metafield(namespace: $namespace, key: $key) { value references(first: 100) { nodes { ... on MediaImage { id image { url } } ... on GenericFile { id url } } } } } }";

pub const MEDIA_POLL: &str = "query MediaPoll($id: ID!) { node(id: $id) { id ... on MediaImage { status fileStatus alt image { url } originalSource { url } mediaErrors { code message details } } } }";

pub const FILE_CREATE: &str = "mutation FileCreate($files: [FileCreateInput!]!) { fileCreate(files: $files) { files { id fileStatus alt } userErrors { field message code } } }";

pub const FILE_POLL: &str = "query FilePoll($id: ID!) { node(id: $id) { id ... on File { fileStatus alt fileErrors { code message details } } ... on MediaImage { image { url } originalSource { url } } ... on GenericFile { url } } }";

/// SV-2 (`sv2_files_by_name`): `filename:<stem>` finds the file and its uniquified copies. P64: the
/// only file lookup (the shop holds tens of thousands of files; it is never listed).
pub const FILES_BY_NAME: &str = "query FilesByName($query: String!) { files(first: 20, query: $query) { nodes { id fileStatus alt ... on MediaImage { image { url } } } } }";

pub const PRODUCT_REORDER_MEDIA: &str = "mutation ProductReorderMedia($id: ID!, $moves: [MoveInput!]!) { productReorderMedia(id: $id, moves: $moves) { job { id done } mediaUserErrors { field message code } } }";

pub const JOB_POLL: &str = "query JobPoll($id: ID!) { job(id: $id) { id done } }";

pub const PRODUCT_DELETE_MEDIA: &str = "mutation ProductDeleteMedia($files: [FileUpdateInput!]!) { fileUpdate(files: $files) { files { id } userErrors { field message code } } }";

pub const FILE_DELETE: &str = "mutation FileDelete($ids: [ID!]!) { fileDelete(fileIds: $ids) { deletedFileIds userErrors { field message code } } }";

pub const DEFINITION_CREATE: &str = "mutation DefinitionCreate($definition: MetafieldDefinitionInput!) { metafieldDefinitionCreate(definition: $definition) { createdDefinition { id namespace key type { name } } userErrors { field message code } } }";

pub const DEFINITION_DELETE: &str = "mutation DefinitionDelete($id: ID!) { metafieldDefinitionDelete(id: $id, deleteAllAssociatedMetafields: true) { deletedDefinitionId userErrors { field message code } } }";

pub const PRODUCT_SET: &str = "mutation ProductSet($input: ProductSetInput!) { productSet(synchronous: true, input: $input) { product { id title status variants(first: 5) { nodes { id sku barcodes(first: 5) { nodes { value type } } inventoryItem { sku } } } } userErrors { field message code } } }";

pub const ITEM_SET_IDENTITY: &str = "mutation ItemSetIdentity($productId: ID!, $variants: [ProductVariantsBulkInput!]!) { productVariantsBulkUpdate(productId: $productId, variants: $variants) { productVariants { id sku barcodes(first: 5) { nodes { value type } } inventoryItem { sku } } userErrors { field message code } } }";

pub const PRODUCT_DELETE: &str = "mutation ProductDelete($id: ID!) { productDelete(input: {id: $id}, synchronous: true) { deletedProductId userErrors { field message } } }";

pub const PRODUCT_UPDATE: &str = "mutation ProductUpdateTags($product: ProductUpdateInput!) { productUpdate(product: $product) { product { id tags title descriptionHtml } userErrors { field message } } }";

pub const PRODUCT_TYPES: &str = "query ProductTypes($after: String) { productTypes(first: 250, after: $after) { pageInfo { hasNextPage endCursor } nodes } }";

/// P65 (changelog 45): the shop's active stock locations (`includeInactive` defaults to false;
/// needs `read_locations`).
pub const LOCATIONS: &str =
    "query Locations { locations(first: 50) { nodes { id name isActive } } }";

/// P65: read-back of a created product's stock at one location (tracking and the `available`
/// quantity of its first variant's inventory item).
pub const INVENTORY_READBACK: &str = "query InventoryReadback($id: ID!, $locationId: ID!) { product(id: $id) { variants(first: 1) { nodes { inventoryItem { tracked inventoryLevel(locationId: $locationId) { quantities(names: [\"available\"]) { name quantity } } } } } } }";

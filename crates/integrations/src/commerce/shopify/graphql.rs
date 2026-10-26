//! Admin GraphQL documents (hand-written, no codegen). These are the exact documents the P12
//! grounding probe (`examples/shopify_probe.rs`) sent to the test store, so every fixture in
//! `fixtures/` is a request/response pair of the document the adapter sends. API 2026-10 has no
//! `productCreateMedia` / `productDeleteMedia`: media are created with `productUpdate(media:)` and
//! removed from a product with `fileUpdate(referencesToRemove:)` (SV-2, SV-4).

pub const DEFINITIONS: &str = "query Definitions($after: String) { metafieldDefinitions(ownerType: PRODUCT, first: 50, after: $after) { pageInfo { hasNextPage endCursor } nodes { id namespace key name description type { name } validations { name value } access { admin } } } }";

pub const PRODUCT_READ: &str = "query ProductRead($id: ID!) { product(id: $id) { id title status tags descriptionHtml variants(first: 5) { nodes { id sku barcodes(first: 5) { nodes { value type } } inventoryItem { sku } } } media(first: 50) { nodes { id status alt mediaContentType ... on MediaImage { image { url } originalSource { url } } } } } }";

pub const VARIANT_SEARCH: &str = "query VariantSearch($query: String!) { productVariants(first: 10, query: $query) { nodes { id sku barcodes(first: 5) { nodes { value } } product { id title status featuredMedia { preview { image { url } } } } } } }";
pub const TITLE_SEARCH: &str = "query TitleSearch($query: String!) { products(first: 20, query: $query) { nodes { id title featuredMedia { preview { image { url } } } variants(first: 1) { nodes { sku barcodes(first: 5) { nodes { value } } inventoryItem { sku } } } } } }";

pub const METAFIELDS_READ: &str = "query MetafieldsRead($id: ID!, $after: String) { product(id: $id) { metafields(first: 100, after: $after) { pageInfo { hasNextPage endCursor } nodes { namespace key type value } } } }";

pub const METAFIELDS_SET: &str = "mutation MetafieldsSet($metafields: [MetafieldsSetInput!]!) { metafieldsSet(metafields: $metafields) { metafields { namespace key type value } userErrors { field message code elementIndex } } }";

pub const STAGED_UPLOADS_CREATE: &str = "mutation StagedUploadsCreate($input: [StagedUploadInput!]!) { stagedUploadsCreate(input: $input) { stagedTargets { url resourceUrl parameters { name value } } userErrors { field message } } }";

pub const PRODUCT_CREATE_MEDIA: &str = "mutation ProductCreateMedia($product: ProductUpdateInput!, $media: [CreateMediaInput!]) { productUpdate(product: $product, media: $media) { product { id newest: media(first: 1, reverse: true) { nodes { id status alt mediaContentType } } all: media(first: 50) { nodes { id status alt } } } userErrors { field message } } }";

pub const MEDIA_POLL: &str = "query MediaPoll($id: ID!) { node(id: $id) { id ... on MediaImage { status fileStatus alt image { url } originalSource { url } mediaErrors { code message details } } } }";

pub const FILE_CREATE: &str = "mutation FileCreate($files: [FileCreateInput!]!) { fileCreate(files: $files) { files { id fileStatus alt } userErrors { field message code } } }";

pub const FILE_POLL: &str = "query FilePoll($id: ID!) { node(id: $id) { id ... on File { fileStatus alt fileErrors { code message details } } ... on MediaImage { image { url } originalSource { url } } ... on GenericFile { url } } }";

pub const FILES_BY_NAME: &str = "query FilesByName($query: String!) { files(first: 20, query: $query) { nodes { id fileStatus alt ... on MediaImage { image { url } } } } }";
pub const FILES_ALL: &str = "query FilesAll($after: String) { files(first: 100, after: $after) { pageInfo { hasNextPage endCursor } nodes { id fileStatus alt ... on MediaImage { image { url } } } } }";

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

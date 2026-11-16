import { previewUrl } from '@/services/sessions';

/** One visual rule for examples, in the editor and later in the session's template info. */
export function TemplateExample({ path }: { path: string | null }) {
  return <div className="template-example" data-bw="true">
    {path ? <img src={previewUrl(path)} alt="Example angle" /> : <span>No example photo</span>}
  </div>;
}

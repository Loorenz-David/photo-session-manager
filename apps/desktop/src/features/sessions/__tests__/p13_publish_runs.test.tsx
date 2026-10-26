import '@testing-library/jest-dom/vitest';
import {cleanup,fireEvent,render,screen,waitFor} from '@testing-library/react';
import {afterEach,expect,it,vi} from 'vitest';

vi.mock('@/services/items',()=>({searchItems:vi.fn(),nextSku:vi.fn(),createItem:vi.fn(),adoptItem:vi.fn()}));
vi.mock('@/services/sessions',async importOriginal=>({...(await importOriginal<typeof import('@/services/sessions')>()),createSession:vi.fn()}));

import {NewSessionDialog} from '@/features/sessions/NewSessionDialog';
import {adoptItem,nextSku,searchItems} from '@/services/items';
import {createSession} from '@/services/sessions';

globalThis.ResizeObserver ??= class {observe(){} unobserve(){} disconnect(){}};
Element.prototype.scrollIntoView ??= function scrollIntoView(){};
afterEach(()=>{cleanup();vi.clearAllMocks()});

it('C9(f) shop search hit is adopted before opening its session',async()=>{
  vi.mocked(nextSku).mockResolvedValue(null);
  vi.mocked(searchItems).mockResolvedValue([{id:0,sku:'K-1',folder_key:'K-1',barcode:null,item_type:'',title:'Oak chair',origin:'linked',external_id:'product-1',created_at:0,updated_at:0}]);
  vi.mocked(adoptItem).mockResolvedValue({id:42,sku:'K-1',folder_key:'K-1',barcode:null,item_type:'',title:'Oak chair',origin:'linked',external_id:null,created_at:0,updated_at:0});
  vi.mocked(createSession).mockResolvedValue({id:7} as Awaited<ReturnType<typeof createSession>>);
  const opened=vi.fn();render(<NewSessionDialog open onOpenChange={()=>undefined} onOpened={opened}/>);
  fireEvent.change(screen.getByLabelText('Search items by title or SKU'),{target:{value:'Oak'}});
  const hit=await screen.findByText('Oak chair');expect(screen.getByText('K-1 · in the shop')).toBeInTheDocument();
  fireEvent.click(hit);
  await waitFor(()=>expect(opened).toHaveBeenCalledWith(7));
  expect(adoptItem).toHaveBeenCalledWith('product-1');expect(createSession).toHaveBeenCalledWith(42);
  expect(vi.mocked(adoptItem).mock.invocationCallOrder[0]).toBeLessThan(vi.mocked(createSession).mock.invocationCallOrder[0]);
});

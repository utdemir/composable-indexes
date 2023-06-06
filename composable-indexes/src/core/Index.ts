import { unreachable } from "../util";
import { AddUpdate, DeleteUpdate, Update, UpdateType, UpdateUpdate, mapUpdate } from "./Update";
import { Id, Item, Store } from "./simple_types";

export {Id, Item, Store}

export abstract class Index<In, Out, Queries> {
  protected constructor(readonly indexContext: IndexContext<Out>) {}
  abstract _onUpdate(update: Update<In>): () => void;
  abstract query: Queries
  protected item(id: Id): Item<Out> {
    return new Item(id, this.indexContext.store.get(id)!);
  }
}

type SomeIndex = Index<any, any, any>
type IndexIn<Ix extends SomeIndex> = Ix extends Index<infer In, any, any> ? In : never
type IndexOut<Ix extends SomeIndex> = Ix extends Index<any, infer Out, any> ? Out : never
type IndexQueries<Ix extends SomeIndex> = Ix extends Index<any, any, infer Queries> ? Queries : never

// UnregisteredIndex

export class IndexContext<Out> {
  constructor(readonly store: Store<Out>) {}
}

export class UnregisteredIndex<Ix extends SomeIndex> {
  constructor(readonly _register: (ctx: IndexContext<IndexOut<Ix>>) => Ix) {}

  focus<NewIn>(
    f: (x: NewIn) => IndexIn<Ix>
  ): UnregisteredIndex<Index<NewIn, IndexOut<Ix>, IndexQueries<Ix>>> {
    return new UnregisteredIndex(ctx => new FocusedIndex(ctx, this._register(ctx), f));
  }

  /** 
  group<Group extends string | number>(
    f: (x: In) => Group
  ): UnregisteredIndex<In, Out, Queries, GroupedIndex<In, Out, Group, Ix>> {
    return GroupedIndex.create(f, this);
  }
  */
}

export function focus<NewIn, Ix extends SomeIndex>(
    f: (_: NewIn) => IndexIn<Ix>,
    inner: UnregisteredIndex<Ix>
): UnregisteredIndex<Index<NewIn, IndexOut<Ix>, IndexQueries<Ix>>> {
    return inner.focus(f)
}

// Focus functionality

class FocusedIndex<
  NewIn,
  Ix extends Index<any, any, any>
> extends Index<NewIn, IndexOut<Ix>, IndexQueries<Ix>> {
  focused: Ix = this.inner;

  constructor(
    ctx: IndexContext<IndexOut<Ix>>,
    private inner: Ix,
    private readonly f: (_: NewIn) => IndexIn<Ix>
  ) {
    super(ctx);
  }

  _onUpdate(update: Update<IndexIn<Ix>>): () => void {
    return this.inner._onUpdate(mapUpdate(this.f, update));
  }

  query: IndexQueries<Ix> = this.inner.query
}

// Group functionality

/*

export class GroupedIndex<In, Out, Group extends string | number, Inner extends Index<In, Out>> extends Index<
  In,
  Out
> {
  private readonly ixs: Map<string | number, Inner> = new Map();

  private constructor(
    private readonly ctx: IndexContext<Out>,
    private readonly inner: UnregisteredIndex<In, Out, Inner>,
    private readonly group: (_: In) => Group
  ) {
    super(ctx);
  }

  static create<In, Out, Group extends string | number, Inner extends Index<In, Out>>(
    f: (_: In) => Group,
    inner: UnregisteredIndex<In, Out, Inner>
  ): UnregisteredIndex<In, Out, GroupedIndex<In, Out, Group, Inner>> {
    return new UnregisteredIndex((ctx: IndexContext<Out>) => {
      const ix = new GroupedIndex(ctx, inner, f);
      return ix;
    });
  }

  _onUpdate(update: Update<In>): () => void {
    if (update.type === UpdateType.ADD) {
      return this.add(update);
    } else if (update.type === UpdateType.UPDATE) {
      return this.update(update);
    } else if (update.type === UpdateType.DELETE) {
      return this.delete(update);
    } else {
      unreachable(update);
    }
  }

  private getOrCreateGroup(group: Group): Inner {
    let ix = this.ixs.get(group);
    if (!ix) {
      ix = this.inner._register(this.ctx);
      this.ixs.set(group, ix);
    }
    return ix
  }
  
  private add(update: AddUpdate<In>): () => void {
    const group = this.group(update.value);
    const ix = this.getOrCreateGroup(group);
    // TODO: If the inner index throws a ConflictException, we should delete the
    // empty index.
    return ix._onUpdate(update);
  }

  private update(update: UpdateUpdate<In>): () => void {
    const oldGroup = this.group(update.oldValue);
    const newGroup = this.group(update.newValue);
    if (oldGroup === newGroup) {
      const ix = this.ixs.get(oldGroup)!;
      return ix._onUpdate(update);
    } else {
      const oldIx = this.ixs.get(oldGroup)!;
      const newIx = this.getOrCreateGroup(newGroup);
      return () => {
        oldIx._onUpdate({
          id: update.id,
          type: UpdateType.DELETE,
          oldValue: update.oldValue,
        })();
        newIx._onUpdate({
          id: update.id,
          type: UpdateType.ADD,
          value: update.newValue,
        })();
      };
    }
  }

  private delete(update: DeleteUpdate<In>): () => void {
    const group = this.group(update.oldValue);
    const ix = this.ixs.get(group)!;
    return ix._onUpdate(update);
    // TODO: When an index becomes empty, we can delete it.
  }

  where<T>(group: string | number): Inner | undefined {
    return this.ixs.get(group);
  }
}

*/

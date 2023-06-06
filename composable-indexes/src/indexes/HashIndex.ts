import { Id } from "..";
import {
  Index,
  IndexContext,
  UnregisteredIndex,
} from "../core/Index";
import { Update, UpdateType } from "../core/Update";
import { Item } from "../core/simple_types";
import { LongSet, unreachable } from "../util";
 
export class HashIndex<In extends number | string, Out> extends Index<In, Out, {
  countDistinct: () => number,
  eq: (value: In) => Item<Out>[],
}> {
  private readonly ix: Map<In, LongSet> = new Map();

  private constructor(ctx: IndexContext<Out>) {
    super(ctx);
  }

  static create<T extends number | string, O>(): UnregisteredIndex<HashIndex<T, O>> {
    return new UnregisteredIndex((ctx) => new HashIndex(ctx));
  }

  _onUpdate(update: Update<In>): () => void {
    return () => {
      if (update.type === UpdateType.ADD) {
        this.add(update.id, update.value);
      } else if (update.type === UpdateType.UPDATE) {
        this.update(update.id, update.oldValue, update.newValue);
      } else if (update.type === UpdateType.DELETE) {
        this.delete(update.id, update.oldValue);
      } else {
        unreachable(update);
      }
    };
  }

  private add(id: Id, value: In): void {
    const set = this.ix.get(value);
    if (set) {
      set.set(id);
    } else {
      const s = new LongSet();
      s.set(id);
      this.ix.set(value, s);
    }
  }

  private update(id: Id, oldValue: In, newValue: In): void {
    this.delete(id, oldValue);
    this.add(id, newValue);
  }

  private delete(id: Id, oldValue: In): void {
    const set = this.ix.get(oldValue);
    set!.delete(id);
    if (set && set.empty()) {
      this.ix.delete(oldValue);
    }
  }

  // Queries
  query = {
    countDistinct: this.countDistinct.bind(this),
    eq: this.eq.bind(this),
  }

  private countDistinct(): number {
    return this.ix.size;
  }

  private eq(value: In): Item<Out>[] {
    return this.items(this.ix.get(value));
  }

  // Utils
  private items(set: LongSet | undefined): Item<Out>[] {
    const ret: Item<Out>[] = [];

    if (!set) return ret;
    set.forEach((id) => {
      ret.push(this.item(id));
    });

    return ret;
  }
}

export function hashIndex<T extends string | number, O>(): UnregisteredIndex<HashIndex<T, O>> {
  return HashIndex.create();
}
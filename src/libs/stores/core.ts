import { get as getStore, writable } from "svelte/store";
import { StorageType } from "./types";
import type { PersistOptions, StorageAdapter, Store, StoreDefinition } from "./types";

/** 存储介质 → 适配器映射表，新增介质只需在此注册 */
const adapters: Record<StorageType, StorageAdapter> = {
  [StorageType.Local]: {
    getItem: (key) => window.localStorage.getItem(key),
    setItem: (key, value) => window.localStorage.setItem(key, value),
    removeItem: (key) => window.localStorage.removeItem(key),
  },
  [StorageType.Session]: {
    getItem: (key) => window.sessionStorage.getItem(key),
    setItem: (key, value) => window.sessionStorage.setItem(key, value),
    removeItem: (key) => window.sessionStorage.removeItem(key),
  },
};

/**
 * 归一化持久化配置：字符串简写 → 完整对象（介质默认 Local）。
 * @param persist 字符串简写或完整持久化配置
 * @returns 完整持久化配置对象
 */
function normalizePersist(persist: string | PersistOptions): PersistOptions {
  return typeof persist === "string" ? { key: persist } : persist;
}

/**
 * 解析初始值。
 * @param initial 初始值，字面量或惰性函数（惰性函数在创建时执行一次）
 * @returns 解析后的初始值
 */
function resolveInitial<T>(initial: T | (() => T)): T {
  return typeof initial === "function" ? (initial as () => T)() : initial;
}

/**
 * 从存储读取持久化数据。
 * @param adapter 存储介质适配器
 * @param key 存储 key
 * @returns 读取结果：found 表示条目是否存在；value 为解析后的数据（条目缺失或损坏时为 undefined）
 */
function loadPersisted(adapter: StorageAdapter, key: string): { found: boolean; value: unknown } {
  try {
    const raw = adapter.getItem(key);
    if (raw == null) return { found: false, value: undefined };
    return { found: true, value: JSON.parse(raw) as unknown };
  } catch {
    // 数据损坏时回退到默认值，并清理损坏条目（避免每次启动重复解析失败）
    try {
      adapter.removeItem(key);
    } catch {
      // 清理失败静默忽略
    }
    return { found: true, value: undefined };
  }
}

/**
 * 创建增强型 store：内存状态始终优先；配置 persist 后 set/update 同步写入存储，写入失败静默忽略；
 * 首次创建时若持久化条目缺失，立即写入默认值。
 * @param initial 初始值，字面量或惰性函数（惰性函数在创建时执行一次）
 * @param options 创建选项
 * @param options.persist 持久化配置：字符串简写默认本地存储，或完整对象指定 key 与介质
 * @param options.subscribe 值变更回调：注册为内部订阅者，创建时与每次值变更（set/update/reset）后触发，接收新值；与持久化相互独立
 * @returns 增强型 store：subscribe 兼容 $store 语法，附带 set/update/get/reset 方法
 */
export function createStore<T>(
  initial: T | (() => T),
  options?: { persist?: string | PersistOptions; subscribe?: (value: T) => void },
): Store<T> {
  const persistOptions = options?.persist ? normalizePersist(options.persist) : null;
  const adapter = persistOptions ? adapters[persistOptions.storage ?? StorageType.Local] : null;
  const defaultValue = resolveInitial(initial);
  const loaded = adapter && persistOptions ? loadPersisted(adapter, persistOptions.key) : null;

  const { subscribe, set } = writable<T>((loaded?.value ?? defaultValue) as T);

  /**
   * 将状态值写入持久化存储。
   * @param value 要写入的状态值
   */
  function persistValue(value: T): void {
    if (!adapter || !persistOptions) return;
    try {
      adapter.setItem(persistOptions.key, JSON.stringify(value));
    } catch {
      // 写入失败（配额满、隐私模式等）不影响内存状态
    }
  }

  if (loaded && !loaded.found) {
    // 首次创建且无历史条目：立即写入默认值，保证存储与内存一致
    persistValue(defaultValue);
  }

  if (options?.subscribe) {
    // 值变更回调注册为内部订阅者：创建时执行一次，此后每次值变更自动触发，与持久化逻辑无关
    subscribe(options.subscribe);
  }

  /** 同步读取当前内存值（update 计算与 get 共用） */
  const getCurrent = () => getStore({ subscribe });

  return {
    subscribe,
    set: (value) => {
      set(value);
      persistValue(value);
    },
    update: (fn) => {
      // 先读后写：fn 保持纯函数（不内嵌持久化副作用），副作用统一在 set 后执行
      const next = fn(getCurrent());
      set(next);
      persistValue(next);
    },
    get: getCurrent,
    reset: () => {
      // 重置语义：清除持久化条目 → 内存重置为默认值。不写回默认——
      // 清除本身即配额失败时的优雅降级（写回失败存储已为空，下次启动按缺失条目自动补写默认）；
      // 用户随后 set/update 时按正常路径持久化
      if (adapter && persistOptions) {
        try {
          adapter.removeItem(persistOptions.key);
        } catch {
          // 删除失败静默忽略
        }
      }
      set(defaultValue);
    },
  };
}

/**
 * 声明子 store 定义：携带精确类型参数，避免字面量变窄丢失联合类型。
 * @param initial 初始值，字面量或惰性函数
 * @param persist 持久化配置（字符串简写默认本地存储）
 * @param subscribe 值变更回调（创建时执行一次，此后每次值变更触发，与持久化相互独立）
 * @returns 子 store 定义（供 createStoreGroup 组合）
 */
export function storeDef<T>(
  initial: T | (() => T),
  persist?: string | PersistOptions,
  subscribe?: (value: T) => void,
): StoreDefinition<T> {
  return { initial, persist, subscribe };
}

/** createStoreGroup 约束用结构子集：仅含协变字段（initial/persist），
 * 规避 subscribe 回调参数逆变导致 `StoreDefinition<T>` 无法赋给 `StoreDefinition<unknown>` 约束 */
type StoreGroupDefinition = {
  initial: unknown | (() => unknown);
  persist?: string | PersistOptions;
};

/**
 * 组合多个子 store 为分组对象：键名即返回对象的属性名，类型逐项映射（属性名为
 * `Store<定义类型>`）。持久化 key 由各定义的 persist 显式指定，与属性名解耦。
 * @param defs 子 store 定义对象（经 storeDef 声明）
 * @returns 分组 store：每个属性为独立的增强型 store（持久化/回退逻辑同 createStore）
 */
export function createStoreGroup<S extends Record<string, StoreGroupDefinition>>(
  defs: S,
): { [K in keyof S]: Store<S[K] extends StoreDefinition<infer T> ? T : never> } {
  // 公开签名已保证类型安全：内部经 Record 装配后统一断言
  const result = {} as Record<string, Store<unknown>>;
  for (const [name, def] of Object.entries(defs)) {
    const definition = def as StoreDefinition<unknown>;
    result[name] = createStore(definition.initial, {
      persist: definition.persist,
      subscribe: definition.subscribe,
    });
  }
  return result as { [K in keyof S]: Store<S[K] extends StoreDefinition<infer T> ? T : never> };
}

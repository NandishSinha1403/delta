import katex from 'katex';
import 'katex/dist/katex.min.css';
import { useMemo } from 'react';
import type { MachineView, ObjectView } from '../engine/api';
import { useCurrentObject } from './current';
import { EngineStatus } from './Status';
import { regexLine, tupleOf, type TupleLine } from './tuple-format';

/** One line of math. KaTeX's HTML is hidden from screen readers, which read `text` instead. */
export function Math({ line, block = false }: { line: TupleLine; block?: boolean }) {
  const html = useMemo(
    () => katex.renderToString(line.tex, { throwOnError: false, output: 'html', displayMode: block }),
    [line.tex, block],
  );
  return (
    <span role="math" aria-label={line.text} className="text-[1.05em]">
      <span aria-hidden="true" dangerouslySetInnerHTML={{ __html: html }} />
    </span>
  );
}

export function DeltaTable({ name, machine }: { name: string; machine: MachineView }) {
  const { columns, rows } = machine.table;
  return (
    <div className="overflow-x-auto">
      <table className="border-collapse font-mono text-sm">
        <caption className="sr-only">
          Transition function δ of {name}. → marks the start state, * marks accept states.
        </caption>
        <thead>
          <tr>
            <th scope="col" className="border-r border-b border-border px-3 py-1.5">
              <span className="sr-only">state</span>
            </th>
            {columns.map((c) => (
              <th key={c} scope="col" className="border-b border-border px-3 py-1.5 font-semibold">
                {c}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => {
            const state = machine.states.find((s) => s.name === row.state);
            return (
              <tr key={row.state} className="hover:bg-surface-2">
                <th scope="row" className="border-r border-border px-3 py-1.5 text-left font-semibold whitespace-nowrap">
                  <span className="inline-block w-4 text-muted" aria-hidden="true">
                    {state?.start ? '→' : ''}
                  </span>
                  <span className="inline-block w-3 text-accept" aria-hidden="true">
                    {state?.accept ? '*' : ''}
                  </span>
                  {row.state}
                  {state?.start && <span className="sr-only"> (start)</span>}
                  {state?.accept && <span className="sr-only"> (accept)</span>}
                </th>
                {row.cells.map((to, i) => (
                  <td key={columns[i] ?? i} className={`px-3 py-1.5 whitespace-nowrap ${to === '∅' ? 'text-muted' : ''}`}>
                    {to}
                  </td>
                ))}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function MachineTuple({ object, machine }: { object: ObjectView; machine: MachineView }) {
  const t = tupleOf(object.name, machine);
  const gnfa = machine.kind === 'gnfa';
  return (
    <>
      <p>
        <Math line={t.header} />, where
      </p>
      <ol className="mt-3 list-decimal space-y-2.5 pl-6 marker:text-muted">
        <li>
          <Math line={t.states} />,
        </li>
        <li>
          <Math line={t.sigma} />,
        </li>
        <li>
          {gnfa ? (
            <Math
              line={{
                tex: '\\delta : (Q - \\{q_{\\text{accept}}\\}) \\times (Q - \\{q_{\\text{start}}\\}) \\to \\mathcal{R}',
                text: 'δ : (Q − {qaccept}) × (Q − {qstart}) → R',
              }}
            />
          ) : (
            <Math line={{ tex: '\\delta', text: 'δ' }} />
          )}{' '}
          is described as
          <div className="mt-2">
            <DeltaTable name={object.name} machine={machine} />
          </div>
        </li>
        <li>
          <Math line={t.start} /> is the start state, and
        </li>
        <li>
          {gnfa ? (
            <>
              <Math line={t.accept} /> is the accept state.
            </>
          ) : (
            <>
              <Math line={t.accept} />.
            </>
          )}
        </li>
      </ol>
      <p className="mt-4 text-xs text-muted">{t.definition}</p>
    </>
  );
}

/** The formal tuple, phrased the way Sipser writes it in Example 1.7. */
export function TuplePanel() {
  const object = useCurrentObject();
  if (!object) return <EngineStatus empty="Declare a dfa, nfa or regex to see its formal definition." />;
  return (
    <section data-testid="tuple" aria-label={`Formal definition of ${object.name}`} className="p-4 text-text">
      {object.derived && <p className="mb-3 font-mono text-xs text-muted">{object.derived}</p>}
      {object.machine ? (
        <MachineTuple object={object} machine={object.machine} />
      ) : (
        <>
          <p className="text-lg" data-testid="regex">
            <Math line={regexLine(object.name, object.regex ?? '')} />
          </p>
          <p className="mt-2 text-sm text-muted">
            over Σ = {'{'}
            {object.sigma.join(', ')}
            {'}'}
          </p>
          <p className="mt-4 text-xs text-muted">Definition 1.52</p>
        </>
      )}
    </section>
  );
}

export function DeltaPanel() {
  const object = useCurrentObject();
  if (!object) return <EngineStatus empty="Declare a machine to see its δ-table." />;
  if (!object.machine) {
    return (
      <p className="p-4 text-sm text-muted">
        {object.name} is a regular expression. Convert it with <code className="font-mono">nfa({object.name})</code> to
        see a δ-table.
      </p>
    );
  }
  return (
    <div className="p-4">
      <DeltaTable name={object.name} machine={object.machine} />
    </div>
  );
}

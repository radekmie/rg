import {
  Button,
  Card,
  ControlGroup,
  Elevation,
  FormGroup,
  HTMLSelect,
  Icon,
  IconName,
  InputGroup,
  Intent,
  Label,
  NumericInput,
} from '@blueprintjs/core';
import {
  Dispatch,
  ReactNode,
  SetStateAction,
  useCallback,
  useEffect,
  useMemo,
  useReducer,
  useState,
} from 'react';
import { AsyncState, usePromise } from 'ui/hooks/usePromise';

import { RgGameDeclaration } from '../../parse';
import { localeCompare, prettyError, random } from '../../utils';
import * as wasm from '../../wasm';
import { useNumericState } from '../hooks/useNumericState';
import * as styles from '../index.module.css';

type BlockProps = { children: ReactNode };
function Block({ children }: BlockProps) {
  return (
    <Card className={styles.block} compact elevation={Elevation.TWO}>
      {children}
    </Card>
  );
}

type InitialState = AsyncState<Awaited<ReturnType<typeof wasm.apply>>, unknown>;
type BenchBlockProps = {
  action: (
    gameDeclaration: RgGameDeclaration,
    initialStatePath: string,
    value: number,
    logger: wasm.Logger,
  ) => Promise<void>;
  actionText: string;
  gameDeclaration: RgGameDeclaration;
  id?: string;
  initialState: InitialState;
  initialStatePath: string;
  initialValue: number;
  label: ReactNode;
  labelInfo?: string;
  logsLimit: number;
  max?: number;
  min?: number;
};

function BenchBlock({
  action,
  actionText,
  gameDeclaration,
  id,
  initialState,
  initialStatePath,
  initialValue,
  label,
  labelInfo,
  logsLimit,
  max,
  min,
}: BenchBlockProps) {
  const inputState = useNumericState(initialValue);

  const [logs, setLogs] = useState<string[]>([]);
  const logger = useMemo(
    () => ({
      log(message: string) {
        message = `[${new Date().toJSON()}] ${message}`;
        setLogs(logs => logs.concat(message).slice(-logsLimit));
      },
    }),
    [logsLimit],
  );

  const [result, setResult] = useState<Intent>(Intent.NONE);
  const disabled =
    result === Intent.PRIMARY ||
    initialState.loading ||
    !initialState.value?.moves.length;

  const onAction = useCallback(() => {
    if (disabled) {
      return;
    }

    async function run() {
      setLogs([]);
      setResult(Intent.PRIMARY);
      try {
        await action(
          gameDeclaration,
          initialStatePath,
          inputState.valueAsNumber,
          logger,
        );
        setResult(Intent.SUCCESS);
      } catch (error) {
        setResult(Intent.DANGER);
        logger.log(prettyError(error));
      }
    }

    // eslint-disable-next-line @typescript-eslint/no-floating-promises -- Run it in background.
    run();
  }, [
    action,
    disabled,
    gameDeclaration,
    initialStatePath,
    inputState.valueAsNumber,
    logger,
  ]);

  return (
    <Block>
      <FormGroup label={label} labelFor={id} labelInfo={labelInfo}>
        <ControlGroup>
          <Button disabled={disabled} intent={result} onClick={onAction}>
            {actionText}
          </Button>
          <NumericInput
            disabled={disabled}
            fill
            id={id}
            max={max}
            min={min}
            onValueChange={inputState.onValueChange}
            value={inputState.valueAsString}
          />
        </ControlGroup>
      </FormGroup>

      <pre>{logs.join('\n')}</pre>
    </Block>
  );
}

type TitleProps = { icon: IconName; text: string };
function Title({ icon, text }: TitleProps) {
  return (
    <>
      <Icon className={styles.title} icon={icon} />
      {text}
    </>
  );
}

export type PlayBlockProps = {
  initialState: InitialState;
  initialStatePath: string;
  initialStatePathActionDispatch: Dispatch<PathAction>;
};

function PlayBlock({
  initialState,
  initialStatePath,
  initialStatePathActionDispatch: dispatch,
}: PlayBlockProps) {
  const [isAuto, setIsAuto] = useState(false);
  useEffect(() => {
    const timeout = setTimeout(() => {
      if (isAuto && initialState.value?.moves.length) {
        dispatch({ kind: 'add', path: random(initialState.value.moves) });
      } else {
        setIsAuto(false);
      }
    });

    return () => clearTimeout(timeout);
  }, [isAuto, initialState.value, dispatch]);

  return (
    <Block>
      <FormGroup
        label={<Title icon="fork" text="Path" />}
        labelFor="path"
        labelInfo="(slash separates moves; space separates tags)"
      >
        <ControlGroup>
          <Button
            icon="double-chevron-left"
            disabled={isAuto || initialStatePath === '/'}
            onClick={() => dispatch({ kind: 'reset' })}
          />
          <Button
            icon="chevron-left"
            disabled={isAuto || initialStatePath === '/'}
            onClick={() => dispatch({ kind: 'pop' })}
          />
          <InputGroup
            disabled={isAuto}
            fill
            id="path"
            onValueChange={path => dispatch({ kind: 'replace', path })}
            value={initialStatePath}
          />
          <HTMLSelect
            disabled={
              isAuto ||
              initialState.loading ||
              !initialState.value?.moves.length
            }
            iconName="caret-down"
            id="move"
            onChange={({ currentTarget: { value } }) =>
              dispatch({ kind: 'add', path: value })
            }
            options={[
              { disabled: true, label: 'Move', value: '(default)' },
              ...(initialState.value?.moves.sort(localeCompare) ?? []),
            ]}
            value="(default)"
          />
          <Button
            icon="random"
            intent={
              isAuto
                ? Intent.PRIMARY
                : initialState.error
                  ? Intent.DANGER
                  : initialState.value?.isFinal
                    ? Intent.SUCCESS
                    : undefined
            }
            disabled={
              isAuto ||
              initialState.loading ||
              !initialState.value?.moves.length
            }
            onClick={() =>
              initialState.value?.moves.length &&
              dispatch({ kind: 'add', path: random(initialState.value.moves) })
            }
          />
          <Button
            icon="double-chevron-right"
            intent={
              isAuto
                ? Intent.PRIMARY
                : initialState.error
                  ? Intent.DANGER
                  : initialState.value?.isFinal
                    ? Intent.SUCCESS
                    : undefined
            }
            disabled={
              isAuto ||
              initialState.loading ||
              !initialState.value?.moves.length
            }
            onClick={() => setIsAuto(true)}
          />
        </ControlGroup>
      </FormGroup>
      <pre>
        {initialState.error
          ? prettyError(initialState.error)
          : initialState.value?.state}
      </pre>
    </Block>
  );
}

type PathAction =
  | { kind: 'add' | 'replace'; path: string }
  | { kind: 'pop' | 'reset' };

function pathReducer(path: string, action: PathAction) {
  switch (action.kind) {
    case 'add':
      return path + action.path;
    case 'pop':
      return path.replace(/\/[^/]*\/$/, '/');
    case 'replace':
      return action.path;
    case 'reset':
      return '/';
  }
}

export type BenchProps = { gameDeclaration: RgGameDeclaration; stats: string };
export function Bench({ gameDeclaration, stats }: BenchProps) {
  const [path, pathActionDispatch] = useReducer(pathReducer, '/');
  const initialState = usePromise(
    () => wasm.apply(gameDeclaration, path),
    [gameDeclaration, path],
  );

  useEffect(() => pathActionDispatch({ kind: 'reset' }), [gameDeclaration]);

  return (
    <section className={styles.wrapScroll}>
      <Block>
        <Label>
          <Title icon="git-repo" text="Resources" />
        </Label>
        <pre>
          <a href="https://ojs.aaai.org/index.php/AAAI/article/view/40203">
            ojs.aaai.org/index.php/AAAI/article/view/40203
          </a>{' '}
          (paper)
          <br />
          <a href="https://github.com/radekmie/rg">
            github.com/radekmie/rg
          </a>{' '}
          (interpreter and tools)
          <br />
          <a href="https://github.com/WoojtekP/RGcompiler">
            github.com/WoojtekP/RGcompiler
          </a>{' '}
          (compiler)
        </pre>
      </Block>
      <Block>
        <Label>
          <Title icon="info-sign" text="Statistics" />
        </Label>
        <pre>{stats}</pre>
      </Block>
      <PlayBlock
        initialState={initialState}
        initialStatePath={path}
        initialStatePathActionDispatch={pathActionDispatch}
      />
      <BenchBlock
        action={wasm.run}
        actionText="Run"
        gameDeclaration={gameDeclaration}
        id="iterations"
        initialState={initialState}
        initialStatePath={path}
        initialValue={100}
        label={<Title icon="generate" text="Iterations" />}
        labelInfo="(number of random games to play)"
        logsLimit={18}
        max={1000}
        min={1}
      />
      <BenchBlock
        action={wasm.perf}
        actionText="Bench"
        gameDeclaration={gameDeclaration}
        id="depth"
        initialState={initialState}
        initialStatePath={path}
        initialValue={3}
        label={<Title icon="diagram-tree" text="Maximum depth" />}
        labelInfo="(game tree depth to calculate)"
        logsLimit={100}
        max={10}
        min={0}
      />
    </section>
  );
}

import { useTranslation } from 'react-i18next'

const en = {
  label: 'Two-player match', timer: 'Phase time remaining', you: '(you)',
  player: { one: 'Player one', two: 'Player two' },
  scoreLabel: 'Round wins for {{player}}',
  lobby: {
    title: 'Meet in the courtyard',
    explanation: 'Create a room and share its invitation, or enter the code from your friend. Win three rounds to take the match.',
    create: 'Create room', or: 'or', code: 'Room code',
    codeHint: 'Use eight letters or digits from the invitation.', join: 'Join room',
  },
  room: {
    invitation: 'Room invitation', codeLabel: 'Room invitation code', players: 'Players',
    ready: 'Ready', cancelReady: 'Not ready', leave: 'Leave room',
    round: 'Round {{round}}', beforeFirstRound: 'Match awaiting first round',
  },
  seat: {
    open: 'Waiting for a player', notReady: 'Not ready', ready: 'Ready', alive: 'Alive',
    eliminated: 'Eliminated', rematchReady: 'Wants a rematch', rematchNotReady: 'Considering rematch',
  },
  stats: {
    label: '{{player}} upgrades', available: 'Bombs available', capacity: 'Bomb capacity',
    range: 'Blast range', speed: 'Speed level',
  },
  phase: {
    waiting: 'Waiting for both players', initialCountdown: 'First round starting',
    interRoundCountdown: 'Next round starting', running: 'Round in progress',
    result: 'Round complete', matchResult: 'Match complete', interrupted: 'Match interrupted',
  },
  roundResult: {
    winner: '{{player}} wins the round.', mutualElimination: 'Draw: both players were eliminated.',
    timeout: 'Draw: time ran out.',
  },
  matchResult: {
    roundWins: '{{player}} wins the match with three round wins.',
    forfeit: '{{player}} wins the match by forfeit.',
    abandoned: 'The match ended after both players left.',
    interrupted: 'The match could not continue: {{reason}}',
  },
  interruption: {
    connectionLost: 'a player lost their connection', serverOverloaded: 'the server is busy',
    serverStopped: 'the server stopped', internalError: 'a server error occurred',
  },
  rematch: {
    ready: 'Play again', withdraw: 'Withdraw rematch vote', locked: 'Rematch opens after the result hold.',
    needsPlayers: 'Both players must be present for a rematch.',
  },
  connection: {
    disconnected: 'Choose how to enter a room.', connecting: 'Opening a secure game session…',
    authenticating: 'Joining the game server…', authenticated: 'Connected. Waiting for the room…',
  },
  arena: {
    label: 'Grenadiers courtyard',
    failure: 'The courtyard could not be drawn. Leave the room and try again.',
    focusHint: 'Select the courtyard to control your player.',
    instructions: 'Move with the arrow keys or W A S D. Place a bomb with Space. Controls work only during a live round.',
  },
  error: {
    title: 'Could not complete that action', connectionFailed: 'The game connection ended. Try again.',
    invalidServerMessage: 'The server sent game data this version cannot use.',
    sessionFailed: 'A game session could not be created. Try again.',
    sequenceExhausted: 'This connection reached its command limit. Start a new session.',
    incompatibleProtocol: 'This game version is not supported by the server.',
    invalidMessage: 'The server could not understand that command.',
    authenticationFailed: 'This game session is no longer valid.', connectionInUse: 'This session is already open elsewhere.',
    notAuthenticated: 'The connection is not ready.', alreadyAuthenticated: 'This connection is already signed in.',
    alreadyInRoom: 'Leave the current room first.', notInRoom: 'Join a room first.',
    roomNotFound: 'That room is no longer available.', roomFull: 'Both seats are occupied.',
    staleRoom: 'The room changed before the command arrived.', invalidPhase: 'That action is unavailable now.',
    staleSequence: 'That command arrived out of order.', duplicateAction: 'That bomb action was already received.',
    rateLimited: 'Commands arrived too quickly.', capacityReached: 'The server has no room for another match.',
  },
} as const

const pl = {
  label: 'Mecz dla dwóch graczy', timer: 'Czas pozostały w bieżącej fazie', you: '(ty)',
  player: { one: 'Gracz pierwszy', two: 'Gracz drugi' },
  scoreLabel: 'Wygrane rundy: {{player}}',
  lobby: {
    title: 'Spotkajcie się na dziedzińcu',
    explanation: 'Utwórz pokój i udostępnij zaproszenie albo wpisz kod otrzymany od znajomego. Wygraj trzy rundy, aby zwyciężyć w meczu.',
    create: 'Utwórz pokój', or: 'lub', code: 'Kod pokoju',
    codeHint: 'Wpisz osiem liter lub cyfr z zaproszenia.', join: 'Dołącz do pokoju',
  },
  room: {
    invitation: 'Zaproszenie do pokoju', codeLabel: 'Kod zaproszenia do pokoju', players: 'Gracze',
    ready: 'Gotowy', cancelReady: 'Jeszcze nie', leave: 'Opuść pokój',
    round: 'Runda {{round}}', beforeFirstRound: 'Oczekiwanie na pierwszą rundę',
  },
  seat: {
    open: 'Oczekiwanie na gracza', notReady: 'Brak gotowości', ready: 'Gotowy', alive: 'W grze',
    eliminated: 'Wyeliminowany', rematchReady: 'Chce rewanżu', rematchNotReady: 'Rozważa rewanż',
  },
  stats: {
    label: 'Ulepszenia: {{player}}', available: 'Dostępne bomby', capacity: 'Limit bomb',
    range: 'Zasięg wybuchu', speed: 'Poziom szybkości',
  },
  phase: {
    waiting: 'Oczekiwanie na obu graczy', initialCountdown: 'Pierwsza runda zaraz się rozpocznie',
    interRoundCountdown: 'Następna runda zaraz się rozpocznie', running: 'Runda trwa',
    result: 'Runda zakończona', matchResult: 'Mecz zakończony', interrupted: 'Mecz przerwany',
  },
  roundResult: {
    winner: '{{player}} wygrywa rundę.', mutualElimination: 'Remis: obaj gracze zostali wyeliminowani.',
    timeout: 'Remis: upłynął czas.',
  },
  matchResult: {
    roundWins: '{{player}} wygrywa mecz trzema rundami.', forfeit: '{{player}} wygrywa mecz walkowerem.',
    abandoned: 'Mecz zakończył się po odejściu obu graczy.',
    interrupted: 'Mecz nie może być kontynuowany: {{reason}}',
  },
  interruption: {
    connectionLost: 'gracz stracił połączenie', serverOverloaded: 'serwer jest przeciążony',
    serverStopped: 'serwer został zatrzymany', internalError: 'wystąpił błąd serwera',
  },
  rematch: {
    ready: 'Zagraj ponownie', withdraw: 'Wycofaj głos za rewanż',
    locked: 'Rewanż będzie dostępny po prezentacji wyniku.',
    needsPlayers: 'Do rewanżu potrzebni są obaj gracze.',
  },
  connection: {
    disconnected: 'Wybierz sposób wejścia do pokoju.', connecting: 'Otwieranie bezpiecznej sesji gry…',
    authenticating: 'Łączenie z serwerem gry…', authenticated: 'Połączono. Oczekiwanie na pokój…',
  },
  arena: {
    label: 'Dziedziniec Grenadiers', failure: 'Nie udało się wyświetlić dziedzińca.',
    focusHint: 'Wybierz dziedziniec, aby sterować graczem.',
    instructions: 'Poruszaj się strzałkami lub klawiszami W A S D. Podłóż bombę spacją. Sterowanie działa tylko podczas rundy.',
  },
  error: {
    title: 'Nie udało się wykonać działania', connectionFailed: 'Połączenie z grą zostało zakończone.',
    invalidServerMessage: 'Serwer wysłał dane nieobsługiwane przez tę wersję.',
    sessionFailed: 'Nie udało się utworzyć sesji gry.', sequenceExhausted: 'Osiągnięto limit poleceń.',
    incompatibleProtocol: 'Serwer nie obsługuje tej wersji gry.', invalidMessage: 'Serwer nie zrozumiał polecenia.',
    authenticationFailed: 'Sesja gry straciła ważność.', connectionInUse: 'Sesja jest otwarta gdzie indziej.',
    notAuthenticated: 'Połączenie nie jest gotowe.', alreadyAuthenticated: 'Połączenie jest już uwierzytelnione.',
    alreadyInRoom: 'Najpierw opuść bieżący pokój.', notInRoom: 'Najpierw dołącz do pokoju.',
    roomNotFound: 'Pokój nie jest już dostępny.', roomFull: 'Oba miejsca są zajęte.',
    staleRoom: 'Pokój zmienił się przed poleceniem.', invalidPhase: 'Ta akcja jest teraz niedostępna.',
    staleSequence: 'Polecenie dotarło w złej kolejności.', duplicateAction: 'Ta bomba została już odebrana.',
    rateLimited: 'Polecenia przychodzą zbyt szybko.', capacityReached: 'Brak miejsca na kolejny mecz.',
  },
} as const

export type V3Copy = typeof en | typeof pl

export function useV3Copy(): V3Copy {
  const { i18n } = useTranslation()
  return i18n.resolvedLanguage?.startsWith('pl') ? pl : en
}

export function format(template: string, values: Record<string, string | number>): string {
  return Object.entries(values).reduce((text, [key, value]) => text.replace(`{{${key}}}`, String(value)), template)
}

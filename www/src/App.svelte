<script>
  const { API_ROOT } = ENV
  let promise = getPending()

  async function getPending() {
      let pending = await(await fetch(`${API_ROOT}/pending`)).json()
      let mods = await(await fetch(`${API_ROOT}/mods`)).json()

      return [pending, mods]
  }

  async function fetchFromSrcom() {
    await fetch(`${API_ROOT}/fetch`)
    promise = getPending()
  }

  function getFlagEmoji(countryCode) {
    const codePoints = countryCode
      .toUpperCase()
      .split('')
      .map(char =>  127397 + char.charCodeAt());
    return String.fromCodePoint(...codePoints);
  }

  function convertDate(date) {
    let dateobj = new Date(date)
    let [y, m, d] = [dateobj.getFullYear(), dateobj.getMonth() + 1, dateobj.getDate()]

    m = m.toString().padStart(2, '0')
    d = d.toString().padStart(2, '0')

    return `${y}-${m}-${d}`
  }

  async function bookRun(run) {
    console.log(`Booking run ${run.id} -> ${run.booked_by}`)

    await fetch(`${API_ROOT}/book/${run.id}/${run.booked_by}`, { method: 'POST' })
  }
</script>

<main>
    <div>
      <button on:click={fetchFromSrcom}>Update runs</button>
    </div>
    {#await promise}
      <p>Loading...</p>
    {:then [pending, mods]}
      <div class="grid">
        <div class="th">Run</div>
        <div class="th">Booked by</div>
        <div class="th">Comment</div>
        {#each pending as run}
          <div>
            <span class="runner">
              <a href="{run.player_url}" target="_blank">
                {run.player_name}
                {getFlagEmoji(run.player_location)}
              </a>
            </span>
            <br/>
            <span class="time">
              <a href="{run.weblink}" target="_blank">{run.times}</a>
            </span>
            <span class="submitted">
              <time datetime={run.submitted}>{convertDate(run.submitted)}</time>
            </span>
          </div>
          <div>
            <select bind:value={run.booked_by} on:change={() => bookRun(run)}>
              {#each mods.Ok as mod}
                <option value="{mod}">{mod}</option>
              {/each}
            </select>
          </div>
          <div class="comment">{run.comment}</div>
        {/each}
      </div>
    {:catch error}
      <p>{error}</p>
    {/await}
</main>

<style>
	main {
    margin: auto;
    padding: 3rem;
    max-width: 800px;
	}

  main > div.grid {
    display: grid;
    grid-template-columns: 1fr 1fr 2fr;
  }

  main > div.grid > div {
    padding: .5em;
  }

  .th {
    background-color: rgb(3, 136, 87);
    padding: .5em;
  }

  .runner {
    font-size: 1.2em;
  }

  .time, .submitted {
    font-size: .9em;
  }

  .run {
    text-align: center;
  }
</style>
